mod config;
mod state;

use log::*;
use sithra_common::{kv, msg, prelude::*};
use sithra_onebot_common::message::*;

const SUBSCRIBERS: &[ioevent::Subscriber<()>] = &[];

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
}
impl From<OneBotGenericId> for GenericId {
    fn from(value: OneBotGenericId) -> Self {
        GenericId::new(kv! {"type": "onebot", "self_id": &value.self_id})
    }
}

#[sithra_common::main(subscribers = SUBSCRIBERS, state = ())]
async fn main(wright: &ioevent::EffectWright) {
    info!("onebot 适配器启动成功");

    let config = match config::OneBotConfig::load() {
        Ok(config) => config,
        Err(e) => {
            error!("加载配置文件失败: {}", e);
            return;
        }
    };
    let generic_id = OneBotGenericId::from_config(&config);
    let ws_api = join_url(&config.ws_url, "/api");
    let ws_event = join_url(&config.ws_url, "/event");
    // 主循环
    loop {
        // 发送事件
        let channel = Channel::new(1234567890, ChannelType::Group);
        let user = User::empty();
        let message = msg!(OneBotMessage[
            at: user.uid.clone(),
            text: "你好",
            img: "file://example.png",
            location: (123.456, 789.012),
        ]);
        let event = MessageEvent::new(generic_id.clone(), channel, user, message);
        let result = wright.emit(&event);
        log::error!("something wrong: {:?}", result);
    }
}

pub fn join_url(url: &str, path: &str) -> String {
    if url.ends_with('/') {
        format!("{}{}", url, path)
    } else {
        format!("{}/{}", url, path)
    }
}
