use crate::{
    error::OneBotApiError,
    internal::api::request::{self, OneBotRequest},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::time::Duration;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

#[derive(Debug, Serialize, Deserialize)]
pub struct OneBotResponse {
    pub status: String,
    pub retcode: i32,
    pub data: Option<Value>,
    pub echo: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OneBotApiClient {
    ws: Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pending_requests: Arc<Mutex<HashMap<String, oneshot::Sender<OneBotResponse>>>>,
}

impl OneBotApiClient {
    pub async fn new(url: &str) -> Result<Self, OneBotApiError> {
        let (ws_stream, _) = connect_async(url).await?;
        let client = Self {
            ws: Arc::new(Mutex::new(ws_stream)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        };

        let ws = client.ws.clone();
        let pending_requests = client.pending_requests.clone();

        tokio::spawn(async move {
            let mut ws = ws.lock().await;
            while let Some(Ok(msg)) = ws.next().await {
                Self::handle_message(msg, &pending_requests).await;
            }
        });

        Ok(client)
    }

    pub async fn call_api<R: OneBotRequest>(
        &self,
        echo: u64,
        params: R,
    ) -> Result<R::RESPONSE, OneBotApiError> {
        let request = request::ApiRequest::new(echo.to_string(), params.into_kind());
        let request_str = serde_json::to_string(&request)?;

        let (tx, rx) = oneshot::channel();
        self.pending_requests
            .lock()
            .await
            .insert(echo.to_string(), tx);

        self.ws
            .lock()
            .await
            .send(tungstenite::Message::Text(request_str.into()))
            .await?;

        let response = tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| OneBotApiError::Timeout)??;

        serde_json::from_value(response.data.unwrap_or(Value::Null))
            .map_err(|_| OneBotApiError::InvalidMessage)
    }

    async fn handle_message(
        msg: tungstenite::Message,
        pending_requests: &Arc<Mutex<HashMap<String, oneshot::Sender<OneBotResponse>>>>,
    ) {
        let msg_str = match msg {
            tungstenite::Message::Text(text) => text,
            _ => return,
        };

        let response: OneBotResponse = match serde_json::from_str(&msg_str) {
            Ok(resp) => resp,
            Err(_) => return,
        };

        let echo = match response.echo.clone() {
            Some(echo) => echo,
            None => return,
        };

        if let Some(sender) = pending_requests.lock().await.remove(&echo) {
            let _ = sender.send(response);
        }
    }
}
