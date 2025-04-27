use crate::{
    error::OneBotApiError,
    internal::api::request::{self, OneBotRequest},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};
use dashmap::DashMap;
use std::num::NonZeroUsize;

// 用于控制并行处理消息的数量
const MAX_CONCURRENT_HANDLERS: usize = 32;

impl From<mpsc::error::SendError<String>> for OneBotApiError {
    fn from(_: mpsc::error::SendError<String>) -> Self {
        OneBotApiError::InvalidMessage
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OneBotResponse {
    pub status: String,
    pub retcode: i32,
    pub data: Option<Value>,
    pub echo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OneBotApiClient {
    pending_requests: Arc<DashMap<String, oneshot::Sender<OneBotResponse>>>,
    tx: mpsc::UnboundedSender<String>,
}

impl OneBotApiClient {
    pub async fn new(url: &str) -> Result<Self, OneBotApiError> {
        let (ws_stream, _) = connect_async(url).await?;
        let (tx, rx) = mpsc::unbounded_channel();
        let pending_requests = Arc::new(DashMap::with_capacity(128));
        
        let client = Self {
            pending_requests: pending_requests.clone(),
            tx,
        };

        // 启动WebSocket处理任务
        tokio::spawn(Self::process_websocket(ws_stream, rx, pending_requests));

        Ok(client)
    }

    async fn process_websocket(
        ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        mut rx: mpsc::UnboundedReceiver<String>,
        pending_requests: Arc<DashMap<String, oneshot::Sender<OneBotResponse>>>,
    ) {
        let (mut ws_writer, mut ws_reader) = ws_stream.split();
        
        // 创建信号量控制并发处理的消息数量
        let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_HANDLERS));
        
        // 处理消息接收的任务
        let receiver_task = {
            let pending_requests = pending_requests.clone();
            let semaphore = semaphore.clone();
            
            tokio::spawn(async move {
                let mut batch = Vec::with_capacity(16);
                let mut interval = tokio::time::interval(Duration::from_millis(5));
                
                loop {
                    tokio::select! {
                        // 收集消息成批处理
                        Some(Ok(msg)) = ws_reader.next() => {
                            batch.push(msg);
                            // 如果批次已满，立即处理
                            if batch.len() >= 16 {
                                let msgs_to_process = std::mem::replace(&mut batch, Vec::with_capacity(16));
                                Self::process_message_batch(msgs_to_process, &pending_requests, &semaphore).await;
                            }
                        }
                        // 定期处理未满的批次
                        _ = interval.tick() => {
                            if !batch.is_empty() {
                                let msgs_to_process = std::mem::replace(&mut batch, Vec::with_capacity(16));
                                Self::process_message_batch(msgs_to_process, &pending_requests, &semaphore).await;
                            }
                        }
                        // 如果WebSocket关闭，退出循环
                        else => break,
                    }
                }
            })
        };
        
        // 处理消息发送的任务
        let sender_task = tokio::spawn(async move {
            // 发送消息批处理
            let mut batch = Vec::with_capacity(16);
            let mut interval = tokio::time::interval(Duration::from_millis(5));
            
            loop {
                tokio::select! {
                    // 收集发送消息
                    Some(msg) = rx.recv() => {
                        batch.push(msg);
                        // 如果批次已满，立即发送
                        if batch.len() >= 16 {
                            let msgs = std::mem::replace(&mut batch, Vec::with_capacity(16));
                            Self::send_message_batch(&mut ws_writer, msgs).await;
                        }
                    }
                    // 定期发送未满的批次
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            let msgs = std::mem::replace(&mut batch, Vec::with_capacity(16));
                            Self::send_message_batch(&mut ws_writer, msgs).await;
                        }
                    }
                    // 如果channel关闭，退出循环
                    else => break,
                }
            }
        });
        
        // 等待任意一个任务结束
        tokio::select! {
            _ = sender_task => {},
            _ = receiver_task => {},
        }
    }
    
    // 批量处理接收消息
    async fn process_message_batch(
        messages: Vec<tungstenite::Message>,
        pending_requests: &Arc<DashMap<String, oneshot::Sender<OneBotResponse>>>,
        semaphore: &Arc<tokio::sync::Semaphore>,
    ) {
        let mut tasks = Vec::with_capacity(messages.len());
        
        for msg in messages {
            // 获取信号量许可
            let permit = match semaphore.clone().acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => return, // 信号量关闭
            };
            
            let pending = pending_requests.clone();
            
            // 创建任务处理单个消息
            let task = tokio::spawn(async move {
                Self::handle_message(msg, pending).await;
                // 当函数返回时，permit会被自动释放
                drop(permit);
            });
            
            tasks.push(task);
        }
        
        // 可选地等待所有任务完成，或者让它们在后台运行
        // futures_util::future::join_all(tasks).await;
    }
    
    // 批量发送消息
    async fn send_message_batch(
        ws_writer: &mut futures_util::stream::SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>,
        messages: Vec<String>,
    ) {
        for msg in messages {
            if let Err(e) = ws_writer.send(tungstenite::Message::Text(msg.into())).await {
                eprintln!("Failed to send message: {}", e);
                // 失败后短暂暂停以避免CPU过度使用
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }

    pub async fn call_api<R: OneBotRequest>(
        &self,
        echo: u64,
        params: R,
    ) -> Result<R::RESPONSE, OneBotApiError> {
        let echo_str = echo.to_string();
        let request = request::ApiRequest::new(echo_str.clone(), params.into_kind());
        
        // 提前准备好所有需要的数据，减少锁定后的操作时间
        let request_str = serde_json::to_string(&request)?;
        let (tx, rx) = oneshot::channel();
        
        // 无需加锁，直接插入到DashMap
        self.pending_requests.insert(echo_str.clone(), tx);

        // 发送请求
        if let Err(e) = self.tx.send(request_str) {
            // 移除失败的请求
            self.pending_requests.remove(&echo_str);
            return Err(e.into());
        }

        // 等待响应，设置超时
        let response = tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| {
                // 超时时清理未完成的请求
                self.pending_requests.remove(&echo_str);
                OneBotApiError::Timeout
            })??;

        // 快速解析响应
        match serde_json::from_value(response.data.unwrap_or(Value::Null)) {
            Ok(result) => Ok(result),
            Err(_) => Err(OneBotApiError::InvalidMessage)
        }
    }

    async fn handle_message(
        msg: tungstenite::Message,
        pending_requests: Arc<DashMap<String, oneshot::Sender<OneBotResponse>>>,
    ) {
        // 提前解析消息，避免在持有锁的情况下执行
        let msg_str = match msg {
            tungstenite::Message::Text(text) => text,
            _ => return,
        };

        let response: OneBotResponse = match serde_json::from_str(&msg_str) {
            Ok(resp) => resp,
            Err(_) => return,
        };

        let echo = match &response.echo {
            Some(echo) => echo,
            None => return,
        };

        // 使用DashMap的无锁特性，直接移除并获取发送者
        if let Some((_, sender)) = pending_requests.remove(echo) {
            let _ = sender.send(response);
        }
    }
}
