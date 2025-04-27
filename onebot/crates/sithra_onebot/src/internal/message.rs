use sithra_onebot_common::message::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum InternalSegment {
    #[serde(rename = "text")]
    Text(TextData),
    #[serde(rename = "image")]
    Image(MediaData),
    #[serde(rename = "record")]
    Record(MediaData),
    #[serde(rename = "at")]
    At(AtData),
    #[serde(rename = "poke")]
    Poke(PokeData),
    #[serde(rename = "share")]
    Share(ShareData),
    #[serde(rename = "contact")]
    Contact(ContactData),
    #[serde(rename = "location")]
    Location(LocationData),
    #[serde(rename = "reply")]
    Reply(ReplyData),
    #[serde(rename = "forward")]
    Forward(ForwardData),
    #[serde(untagged)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TextData {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MediaData {
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtData {
    pub id: Option<String>,
    pub qq: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PokeData {
    #[serde(rename = "type")]
    pub poke_type: String,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ShareData {
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContactData {
    #[serde(rename = "type")]
    pub contact_type: ContactType,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LocationData {
    pub lat: String,
    pub lon: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReplyData {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ContactType {
    QQ,
    Group,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardData {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalForwardMessage {
    #[serde(rename = "type")]
    pub r#type: String,
    pub data: InternalForwardMessageData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalForwardMessageData {
    pub user_id: String,
    pub nickname: String,
    pub content: Vec<InternalSegment>,
}

impl InternalForwardMessage {
    pub fn new(user_id: u64, nickname: String, content: Vec<InternalSegment>) -> Self {
        Self {
            r#type: "node".to_string(),
            data: InternalForwardMessageData {
                user_id: user_id.to_string(),
                nickname,
                content,
            },
        }
    }
}

impl From<InternalSegment> for OneBotSegment {
    fn from(value: InternalSegment) -> Self {
        match value {
            InternalSegment::Text(data) => OneBotSegment::text(data.text),
            InternalSegment::Image(data) => OneBotSegment::img(data.file),
            InternalSegment::Record(data) => OneBotSegment::record(data.file),
            InternalSegment::At(data) => {
                if let Some(id) = data.id {
                    OneBotSegment::at(id.as_str())
                } else if let Some(qq) = data.qq {
                    OneBotSegment::at(qq.as_str())
                } else {
                    OneBotSegment::text("[@]")
                }
            }
            InternalSegment::Poke(data) => OneBotSegment::poke(data.id.as_str()),
            InternalSegment::Location(data) => {
                let lat = data.lat.parse().unwrap_or(0.0);
                let lon = data.lon.parse().unwrap_or(0.0);
                OneBotSegment::location((lat, lon))
            }
            InternalSegment::Reply(data) => OneBotSegment::reply(data.id.as_str()),
            InternalSegment::Forward(data) => OneBotSegment::forward(ForwardId::new(data.id)),
            _ => OneBotSegment::Unknown,
        }
    }
}

impl From<OneBotSegment> for InternalSegment {
    fn from(value: OneBotSegment) -> Self {
        match value {
            OneBotSegment::Text(text) => InternalSegment::Text(TextData { text }),
            OneBotSegment::Image(url) => InternalSegment::Image(MediaData { file: url }),
            OneBotSegment::At(user_id) => InternalSegment::At(AtData {
                id: Some(user_id.to_string()),
                qq: None,
            }),
            OneBotSegment::Record(url) => InternalSegment::Record(MediaData { file: url }),
            OneBotSegment::Poke(user_id) => InternalSegment::Poke(PokeData {
                poke_type: "poke".to_string(),
                id: user_id.to_string(),
            }),
            OneBotSegment::Location { lat, lon } => InternalSegment::Location(LocationData {
                lat: lat.to_string(),
                lon: lon.to_string(),
            }),
            OneBotSegment::Reply(message_id) => InternalSegment::Reply(ReplyData {
                id: message_id.to_string(),
            }),
            OneBotSegment::Forward(forward_id) => {
                InternalSegment::Forward(ForwardData { id: forward_id.to_string() })
            }
            OneBotSegment::Unknown => InternalSegment::Unknown,
        }
    }
}
