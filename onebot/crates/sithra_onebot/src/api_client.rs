use tokio::sync::oneshot;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream, tungstenite};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::OneBotApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct OneBotResponse {
    pub status: String,
    pub retcode: i32,
    pub data: Option<Value>,
    pub echo: Option<String>,
}

pub struct OneBotClient {
    ws: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<OneBotResponse>>>>,
}

impl OneBotClient {
    pub async fn new(url: &str) -> Result<Self, OneBotApiError> {
        let (ws_stream, _) = connect_async(url).await?;
        let client = Self {
            ws: Arc::new(Mutex::new(ws_stream)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 启动消息处理循环
        let ws = client.ws.clone();
        let pending_requests = client.pending_requests.clone();
        tokio::spawn(async move {
            let mut ws = ws.lock().await;
            while let Some(msg) = ws.next().await {
                if let Ok(msg) = msg {
                    Self::handle_message(msg, &pending_requests).await;
                }
            }
        });

        Ok(client)
    }

    async fn handle_message(
        msg: tungstenite::Message,
        pending_requests: &Arc<Mutex<HashMap<String, oneshot::Sender<OneBotResponse>>>>
    ) {
        let msg_str = match msg {
            tungstenite::Message::Text(text) => text,
            _ => return,
        };

        let response = match serde_json::from_str::<OneBotResponse>(&msg_str) {
            Ok(resp) => resp,
            Err(_) => return,
        };

        let echo = match &response.echo {
            Some(echo) => echo.clone(),
            None => return,
        };

        if let Some(sender) = pending_requests.lock().await.remove(&echo) {
            let _ = sender.send(response);
        }
    }

    pub async fn call_api(&self, action: &str, params: Value) -> Result<OneBotResponse, OneBotApiError> {
        let echo = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
            
        let request = serde_json::json!({
            "action": action,
            "params": params,
            "echo": echo
        });

        let (tx, rx) = oneshot::channel();
        self.pending_requests.lock().await.insert(echo.clone(), tx);

        let mut ws = self.ws.lock().await;
        let request_str = request.to_string();
        ws.send(tungstenite::Message::Text(request_str.into())).await?;

        Ok(rx.await?)
    }
} 