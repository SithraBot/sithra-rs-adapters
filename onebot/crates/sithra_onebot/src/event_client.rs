use crate::internal::event::InternalOnebotEvent;
use crate::error::OneBotApiError;
use futures_util::StreamExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite};

#[derive(Debug)]
pub struct OneBotEventClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl OneBotEventClient {
    pub async fn new(url: &str) -> Result<Self, OneBotApiError> {
        let (ws_stream, _) = connect_async(url).await?;
        Ok(Self {
            ws: ws_stream,
        })
    }

    pub async fn recv(&mut self) -> Result<Option<InternalOnebotEvent>, OneBotApiError> {
        if let Some(msg) = self.ws.next().await {
            match msg {
                Ok(tungstenite::Message::Text(text)) => {
                    serde_json::from_str::<InternalOnebotEvent>(&text)
                        .map(Some)
                        .map_err(OneBotApiError::Json)
                }
                Ok(_) => Err(OneBotApiError::InvalidMessage),
                Err(e) => Err(OneBotApiError::WebSocket(e)),
            }
        } else {
            Ok(None)
        }
    }
} 