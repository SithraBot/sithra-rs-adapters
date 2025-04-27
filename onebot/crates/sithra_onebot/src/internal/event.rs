use ioevent::Event;
use serde::{Deserialize, Serialize};

use super::message::InternalSegment;

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalOnebotEvent {
    pub time: u64,
    pub self_id: u64,
    #[serde(flatten)]
    pub kind: InternalOnebotEventKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "post_type")]
pub enum InternalOnebotEventKind {
    #[serde(rename = "message")]
    Message(InternalMessageEvent),
    #[serde(rename = "notice")]
    Notice(InternalNoticeEvent),
    #[serde(rename = "request")]
    Request(InternalRequestEvent),
    #[serde(rename = "meta_event")]
    Meta(InternalMetaEvent),
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

/* 消息事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum InternalMessageEvent {
    #[serde(rename = "private")]
    Private(InternalPrivateMessage),
    #[serde(rename = "group")]
    Group(InternalGroupMessage),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalPrivateMessage {
    pub sub_type: String,
    pub message_id: i32,
    pub user_id: u64,
    pub message: Vec<InternalSegment>,
    pub raw_message: String,
    pub font: i32,
    pub sender: InternalPrivateSender,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalPrivateSender {
    pub user_id: u64,
    pub nickname: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalGroupMessage {
    pub sub_type: String,
    pub message_id: i32,
    pub group_id: u64,
    pub user_id: u64,
    pub anonymous: Option<InternalAnonymous>,
    pub message: Vec<InternalSegment>,
    pub raw_message: String,
    pub font: i32,
    pub sender: InternalGroupSender,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalGroupSender {
    pub user_id: u64,
    pub nickname: Option<String>,
    pub card: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InternalAnonymous {
    pub id: u64,
    pub name: String,
    pub flag: String,
}

/* 通知事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "notice_type")]
pub enum InternalNoticeEvent {
    #[serde(rename = "group_upload")]
    GroupUpload(InternalGroupUploadNotice),
    #[serde(rename = "group_admin")]
    GroupAdmin(InternalGroupAdminNotice),
    #[serde(rename = "group_decrease")]
    GroupDecrease(InternalGroupDecreaseNotice),
    #[serde(rename = "group_increase")]
    GroupIncrease(InternalGroupIncreaseNotice),
    #[serde(rename = "group_ban")]
    GroupBan(InternalGroupBanNotice),
    #[serde(rename = "friend_add")]
    FriendAdd(InternalFriendAddNotice),
    #[serde(rename = "group_recall")]
    GroupRecall(InternalGroupRecallNotice),
    #[serde(rename = "friend_recall")]
    FriendRecall(InternalFriendRecallNotice),
    #[serde(rename = "notify")]
    Notify(NotifyEvent),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupUploadNotice {
    pub group_id: u64,
    pub user_id: u64,
    pub file: InternalFileInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalFileInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub busid: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupAdminNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupDecreaseNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupIncreaseNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupBanNotice {
    pub sub_type: String,
    pub group_id: u64,
    pub operator_id: u64,
    pub user_id: u64,
    pub duration: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalFriendAddNotice {
    pub user_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupRecallNotice {
    pub group_id: u64,
    pub user_id: u64,
    pub operator_id: u64,
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalFriendRecallNotice {
    pub user_id: u64,
    pub message_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, Event)]
#[serde(tag = "sub_type")]
pub enum NotifyEvent {
    #[serde(rename = "poke")]
    Poke {
        group_id: u64,
        user_id: u64,
        target_id: u64,
    },
    #[serde(rename = "lucky_king")]
    LuckyKing {
        group_id: u64,
        user_id: u64,
        target_id: u64,
    },
    #[serde(rename = "honor")]
    Honor {
        group_id: u64,
        honor_type: String,
        user_id: u64,
    },
    #[serde(other)]
    Unknown,
}
/* 请求事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "request_type")]
pub enum InternalRequestEvent {
    #[serde(rename = "friend")]
    Friend(InternalFriendRequest),
    #[serde(rename = "group")]
    Group(InternalGroupRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalFriendRequest {
    pub user_id: u64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InternalGroupRequest {
    pub sub_type: String,
    pub group_id: u64,
    pub user_id: u64,
    pub comment: String,
    pub flag: String,
}

/* 元事件 */
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "meta_event_type")]
pub enum InternalMetaEvent {
    #[serde(rename = "lifecycle")]
    Lifecycle { sub_type: String },
    #[serde(rename = "heartbeat")]
    // TODO: 其他未知元事件类型
    #[serde(other)]
    Unknown,
}