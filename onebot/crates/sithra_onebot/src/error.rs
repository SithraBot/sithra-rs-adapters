use thiserror::Error;

#[derive(Debug, Error)]
pub enum OneBotApiError {
    #[error("WebSocket连接错误: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON序列化错误: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("通道错误: {0}")]
    ChannelError(#[from] tokio::sync::oneshot::error::RecvError),

    #[error("WebSocket消息类型错误")]
    InvalidMessageType,

    #[error("响应超时")]
    Timeout,

    #[error("未知错误: {0}")]
    Unknown(String),
} 