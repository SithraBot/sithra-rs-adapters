use futures_util::{
    StreamExt,
    stream::{SplitSink, SplitStream},
};
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message as WsMessage};

pub struct OneBotState {
    api_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, WsMessage>,
}
impl OneBotState {
    pub async fn new(
        api_url: &str,
    ) -> Result<
        (
            Self,
            SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
        ),
        OneBotConnectError,
    > {
        let (ws_stream, _) = tokio_tungstenite::connect_async(api_url).await?;
        let (api_sender, api_receiver) = ws_stream.split();
        Ok((Self { api_sender }, api_receiver))
    }
}

#[derive(Debug, Error)]
pub enum OneBotConnectError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("WebSocket error: {0}")]
    Ws(#[from] tokio_tungstenite::tungstenite::Error),
}
