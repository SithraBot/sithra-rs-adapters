use serde::{Deserialize, Serialize};
use sithra_common::kv;
use sithra_common::message::*;
use sithra_common::model::*;
use sithra_common::vec;
/// 一般消息段类型
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum OneBotSegment {
    /// 文本(文本内容)
    Text(String),
    /// 图片(图片 URL)
    Image(String),
    /// 提及用户(用户 ID)
    At(UserId),
    /// QQ 表情(QQ 表情 ID)
    Face(String),
    /// 语音(语音 URL)
    Record(String),
    /// 群聊戳一戳
    Poke(UserId),
    /// 位置
    Location { lat: f64, lon: f64 },
    /// 回复
    Reply(MessageId),
    /// 合并转发
    Forward(ForwardId),
    /// 未知消息段
    Unknown(SegmentRaw),
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ForwardId(String);
impl OneBotSegment {
    /// 生成文本消息段
    pub fn text<S: ToString>(text: S) -> Self {
        Self::Text(text.to_string())
    }
    /// 生成图片消息段
    pub fn img<S: ToString>(url: S) -> Self {
        Self::Image(url.to_string())
    }
    /// 生成提及用户消息段
    pub fn at<S: Into<UserId>>(user_id: S) -> Self {
        Self::At(user_id.into())
    }
    /// 生成 QQ 表情消息段
    pub fn face<S: ToString>(id: S) -> Self {
        Self::Face(id.to_string())
    }
    /// 生成语音消息段
    pub fn record<S: ToString>(url: S) -> Self {
        Self::Record(url.to_string())
    }
    /// 生成群聊戳一戳消息段
    pub fn poke<S: Into<UserId>>(user_id: S) -> Self {
        Self::Poke(user_id.into())
    }
    /// 生成位置消息段
    pub fn location<A: Into<f64>, B: Into<f64>>((lat, lon): (A, B)) -> Self {
        Self::Location {
            lat: lat.into(),
            lon: lon.into(),
        }
    }
    /// 生成回复消息段
    pub fn reply<S: Into<MessageId>>(message_id: S) -> Self {
        Self::Reply(message_id.into())
    }
    /// 生成合并转发消息段
    pub fn forward<S: Into<ForwardId>>(forward_id: S) -> Self {
        Self::Forward(forward_id.into())
    }
    /// 生成未知消息段
    pub fn unknown<S: ToString>(r#type: S, kv: KV) -> Self {
        Self::Unknown(SegmentRaw::new(r#type.to_string(), kv))
    }
}
impl FromRawSegment for OneBotSegment {
    fn from_raw_segment(segment: &mut SegmentRaw) -> Option<Self> {
        match segment.r#type.as_str() {
            "text" => Some(OneBotSegment::Text(segment.kv.remove("content")?)),
            "image" => Some(OneBotSegment::Image(segment.kv.remove("url")?)),
            "at" => Some(OneBotSegment::At(UserId::new(
                segment.kv.remove("user_id")?,
            ))),
            "face" => Some(OneBotSegment::Face(segment.kv.remove("id")?)),
            "record" => Some(OneBotSegment::Record(segment.kv.remove("url")?)),
            "poke" => Some(OneBotSegment::Poke(UserId::new(
                segment.kv.remove("user_id")?,
            ))),
            "location" => Some(OneBotSegment::Location {
                lat: segment.kv.remove("lat")?.parse().ok()?,
                lon: segment.kv.remove("lon")?.parse().ok()?,
            }),
            "reply" => Some(OneBotSegment::Reply(MessageId::new(
                segment.kv.remove("message_id")?,
            ))),
            "forward" => Some(OneBotSegment::Forward(ForwardId(segment.kv.remove("id")?))),
            _ => Some(OneBotSegment::Unknown(segment.clone())),
        }
    }
}
impl Segment for OneBotSegment {
    type Serializer = OneBotMessageSerializer;
    type Deserializer = OneBotSegment;
}
/// 一般消息类型。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OneBotMessage {
    /// 消息 ID
    id: Option<MessageId>,
    /// 消息段
    inner: SVec<OneBotSegment>,
}
pub struct OneBotMessageSerializer;
impl MessageSerializer for OneBotMessageSerializer {
    type Input = OneBotSegment;
    fn serialize(message: Self::Input) -> Option<SegmentRaw> {
        match message {
            OneBotSegment::Text(text) => Some(SegmentRaw::text(text)),
            OneBotSegment::Image(url) => Some(SegmentRaw::img(url)),
            OneBotSegment::At(user_id) => Some(SegmentRaw::at(user_id.to_string())),
            OneBotSegment::Face(id) => Some(SegmentRaw::new("face", kv! { "id": &id })),
            OneBotSegment::Record(url) => Some(SegmentRaw::new("record", kv! { "url": &url })),
            OneBotSegment::Poke(user_id) => Some(SegmentRaw::new(
                "poke",
                kv! { "user_id": &user_id.to_string() },
            )),
            OneBotSegment::Location { lat, lon } => Some(SegmentRaw::new(
                "location",
                kv! { "lat": &lat.to_string(), "lon": &lon.to_string() },
            )),
            OneBotSegment::Reply(message_id) => Some(SegmentRaw::new(
                "reply",
                kv! { "message_id": &message_id.to_string() },
            )),
            OneBotSegment::Forward(forward_id) => {
                Some(SegmentRaw::new("forward", kv! { "id": &forward_id.0 }))
            }
            OneBotSegment::Unknown(segment) => Some(segment),
        }
    }
}
impl MessageDeserializer for OneBotMessageSerializer {
    type Output = OneBotSegment;
    fn deserialize(mut segment: SegmentRaw) -> Option<Self::Output> {
        let kind = OneBotSegment::from_raw_segment(&mut segment)?;
        Some(kind)
    }
}
impl IntoIterator for OneBotMessage {
    type Item = OneBotSegment;
    type IntoIter = vec::IntoIter<[Self::Item; 3]>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
impl Message for OneBotMessage {
    type Segment = OneBotSegment;
    fn id(&self) -> Option<MessageId> {
        self.id.clone()
    }
    fn from_raw(raw: MessageRaw) -> Self {
        let segments = Self::segments(raw.segments).collect();
        Self {
            id: raw.id,
            inner: segments,
        }
    }
    fn from_array<const N: usize>(array: [Self::Segment; N]) -> Self {
        let segments = array.into_iter().collect();
        Self {
            id: None,
            inner: segments,
        }
    }
}
