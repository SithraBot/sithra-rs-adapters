mod api_client;
mod config;
mod error;
mod event_client;
mod internal;
mod procedure;
mod state;
mod subscribers;

use internal::event::{
    InternalGroupMessage, InternalMessageEvent, InternalOnebotEventKind, InternalPrivateMessage,
};
use log::*;
use sithra_common::{kv, prelude::*};
use sithra_onebot_common::message::{OneBotMessage, OneBotSegment};
use state::OneBotAdapterState;
use std::sync::Arc;
use subscribers::SUBSCRIBERS;

#[derive(Debug, Clone)]
pub struct OneBotGenericId {
    self_id: String,
}
impl OneBotGenericId {
    pub fn from_config(config: &config::OneBotConfig) -> Self {
        Self {
            self_id: config.self_id.clone(),
        }
    }
}
impl EnsureGenericId for OneBotGenericId {
    type Error = String;
    fn ensure_generic_id(id: &GenericId) -> Result<Self, Self::Error> {
        let self_id = id.get("self_id").ok_or("self_id not found")?;
        Ok(OneBotGenericId {
            self_id: self_id.to_string(),
        })
    }
    fn match_adapter(id: &GenericId) -> bool {
        let self_id = id.get("self_id");
        let ty = id.get("type");
        if self_id != Some(&"onebot".to_string()) {
            return false;
        }
        if ty != Some(&"onebot".to_string()) {
            return false;
        }
        true
    }
}
impl From<OneBotGenericId> for GenericId {
    fn from(value: OneBotGenericId) -> Self {
        GenericId::new(kv! {"type": "onebot", "self_id": &value.self_id})
    }
}

fn build_onebot_message(
    message_id: i32,
    segments: SVec<internal::message::InternalSegment>,
) -> OneBotMessage {
    let mut msg = SVec::<OneBotSegment>::new();
    for segment in segments {
        msg.push(segment.into());
    }
    OneBotMessage::new(Some(MessageId::new(message_id)), msg)
}

fn build_user(user_id: u64, nickname: Option<String>, card: Option<String>) -> User {
    User::new(
        user_id,
        nickname.unwrap_or("Unknown".to_string()),
        card,
        None,
    )
}

async fn handle_message(
    wright: &ioevent::EffectWright,
    generic_id: &OneBotGenericId,
    message_id: i32,
    segments: SVec<internal::message::InternalSegment>,
    user_id: u64,
    nickname: Option<String>,
    card: Option<String>,
    channel_id: u64,
    channel_type: ChannelType,
) -> Result<(), error::OneBotApiError> {
    let msg = build_onebot_message(message_id, segments);
    let channel = Channel::new(channel_id, channel_type);
    let user = build_user(user_id, nickname, card);
    let event = MessageEvent::new(generic_id.clone(), channel, user, msg);

    wright
        .emit(&event)
        .map_err(|e| error::OneBotApiError::Internal(e.to_string()))
}

async fn handle_private_message(
    wright: &ioevent::EffectWright,
    generic_id: &OneBotGenericId,
    message: InternalPrivateMessage,
) -> Result<(), error::OneBotApiError> {
    handle_message(
        wright,
        generic_id,
        message.message_id,
        message.message,
        message.user_id,
        message.sender.nickname,
        None,
        message.user_id,
        ChannelType::Private,
    )
    .await
}

async fn handle_group_message(
    wright: &ioevent::EffectWright,
    generic_id: &OneBotGenericId,
    message: InternalGroupMessage,
) -> Result<(), error::OneBotApiError> {
    handle_message(
        wright,
        generic_id,
        message.message_id,
        message.message,
        message.user_id,
        message.sender.nickname,
        message.sender.card,
        message.group_id,
        ChannelType::Group,
    )
    .await
}

async fn process_events(
    mut event_client: event_client::OneBotEventClient,
    wright: Arc<ioevent::EffectWright>,
    generic_id: Arc<OneBotGenericId>,
) {
    while let Ok(Some(event)) = event_client.recv().await {
        let event_kind = event.kind;
        let wright = wright.clone();
        let generic_id = generic_id.clone();

        tokio::spawn(async move {
            match event_kind {
                InternalOnebotEventKind::Message(message) => match message {
                    InternalMessageEvent::Private(msg) => {
                        if let Err(e) = handle_private_message(&wright, &generic_id, msg).await {
                            error!("处理私聊消息失败: {}", e);
                        }
                    }
                    InternalMessageEvent::Group(msg) => {
                        if let Err(e) = handle_group_message(&wright, &generic_id, msg).await {
                            error!("处理群消息失败: {}", e);
                        }
                    }
                },
                _ => {} // TODO: 处理其他事件
            }
        });
    }

    info!("WebSocket连接已断开");
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = OneBotAdapterState::new().await)]
async fn main(wright: &ioevent::EffectWright) {
    info!("onebot 适配器启动成功");

    let config = match config::OneBotConfig::load() {
        Ok(config) => config,
        Err(e) => {
            error!("加载配置文件失败: {}", e);
            return;
        }
    };

    let generic_id = Arc::new(OneBotGenericId::from_config(&config));
    let wright = Arc::new(wright.clone());
    let ws_event = join_url(&config.ws_url, "/event");

    loop {
        info!("正在连接事件WebSocket...");
        let event_client = match event_client::OneBotEventClient::new(&ws_event).await {
            Ok(client) => client,
            Err(e) => {
                error!("连接事件WebSocket失败: {}", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        info!("WebSocket连接成功");
        process_events(event_client, wright.clone(), generic_id.clone()).await;
    }
}

pub fn join_url(url: &str, path: &str) -> String {
    if url.ends_with('/') {
        format!("{}{}", url, path)
    } else {
        format!("{}/{}", url, path)
    }
}
