use ioevent::error::CallSubscribeError;
use thiserror::Error;
use tokio_tungstenite::tungstenite;
use tokio::sync::oneshot;

#[derive(Debug, Error)]
pub enum OneBotApiError {
    #[error("WebSocket错误: {0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("通道错误: {0}")]
    Channel(#[from] oneshot::error::RecvError),

    #[error("请求超时")]
    Timeout,

    #[error("消息格式错误")]
    InvalidMessage,

    #[error("内部错误: {0}")]
    Internal(String),
}

impl OneBotApiError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            OneBotApiError::WebSocket(_) | 
            OneBotApiError::Timeout
        )
    }
}

impl From<OneBotApiError> for CallSubscribeError {
    fn from(error: OneBotApiError) -> Self {
        CallSubscribeError::Other(error.to_string())
    }
}
