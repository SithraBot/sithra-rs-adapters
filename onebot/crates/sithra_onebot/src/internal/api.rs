pub mod response {
    use crate::internal::message::InternalSegment;
    use serde::{Deserialize, Serialize};
    /// API响应基础结构
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse {
        /// 响应状态
        pub status: Option<String>,
        /// 返回码
        pub retcode: Option<i32>,
        /// 响应数据
        pub data: Option<ApiResponseKind>,
        /// 与请求对应的echo标识
        pub echo: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct OnlyEcho {
        pub echo: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(untagged)]
    pub enum ApiResponseKind {
        MessageIdResponse(MessageIdResponse),
        MessageDetail(MessageDetail),
        StrangerInfo(StrangerInfo),
        GroupInfo(GroupInfo),
        GroupMemberList(GroupMemberList),
        StatusInfo(StatusInfo),
        VersionInfo(VersionInfo),
        LoginInfo(LoginInfo),
        GroupMemberInfo(GroupMemberInfo),
        ForwardIdResponse(ForwardIdResponse),
        Unknown(serde_json::Value),
    }

    /// 消息发送响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MessageIdResponse {
        /// 消息ID（用于撤回等功能）
        pub message_id: i32,
    }

    /// 消息详情响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct MessageDetail {
        /// 消息发送时间戳
        pub time: i64,
        /// 消息类型（private/group）
        pub message_type: String,
        /// 消息ID
        pub message_id: i32,
        /// 消息真实ID
        pub real_id: i32,
        /// 发送者信息
        pub sender: SenderInfo,
        /// 消息内容（已解析的消息段）
        pub message: Vec<InternalSegment>,
    }

    /// 发送者信息结构
    #[derive(Debug, Serialize, Deserialize)]
    pub struct SenderInfo {
        /// 用户QQ号
        pub user_id: i64,
        /// 昵称
        pub nickname: String,
        /// 群角色（仅群消息有效）
        #[serde(skip_serializing_if = "Option::is_none")]
        pub role: Option<String>,
        /// 群名片（仅群消息有效）
        #[serde(skip_serializing_if = "Option::is_none")]
        pub card: Option<String>,
    }

    /// 登录信息响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct LoginInfo {
        /// 当前登录的QQ号
        pub user_id: i64,
        /// 当前登录的昵称
        pub nickname: String,
    }

    /// 陌生人信息响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct StrangerInfo {
        /// QQ号
        pub user_id: i64,
        /// 昵称
        pub nickname: String,
        /// 性别（male/female/unknown）
        pub sex: Option<String>,
        /// 年龄
        pub age: i32,
        /// 地区信息
        #[serde(skip_serializing_if = "Option::is_none")]
        pub area: Option<String>,
    }

    /// 群信息响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GroupInfo {
        /// 群号
        pub group_id: i64,
        /// 群名称
        pub group_name: String,
        /// 当前成员数量
        pub member_count: Option<i32>,
        /// 最大成员数
        pub max_member_count: Option<i32>,
    }

    /// 群成员信息响应数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GroupMemberInfo {
        /// 群号
        pub group_id: i64,
        /// 用户QQ号
        pub user_id: i64,
        /// 用户昵称
        pub nickname: String,
        /// 群名片
        pub card: Option<String>,
        /// 性别
        pub sex: Option<String>,
        /// 年龄
        pub age: Option<i32>,
        /// 地区
        pub area: Option<String>,
        /// 加群时间戳
        pub join_time: i64,
        /// 最后发言时间
        pub last_sent_time: i64,
        /// 成员等级
        pub level: Option<String>,
        /// 角色（owner/admin/member）
        pub role: Option<String>,
        /// 专属头衔
        #[serde(skip_serializing_if = "Option::is_none")]
        pub title: Option<String>,
    }

    /// 群成员列表响应数据（JSON数组包装）
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GroupMemberList(pub Vec<GroupMemberInfo>);

    /// 运行状态响应
    #[derive(Debug, Serialize, Deserialize)]
    pub struct StatusInfo {
        /// 是否在线（null表示未知）
        pub online: Option<bool>,
        /// 状态是否正常
        pub good: bool,
    }

    /// 版本信息响应
    #[derive(Debug, Serialize, Deserialize)]
    pub struct VersionInfo {
        /// 实现名称（如go-cqhttp）
        pub app_name: String,
        /// 实现版本
        pub app_version: String,
        /// 协议版本（如v11）
        pub protocol_version: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ForwardIdResponse(pub String);
}

pub mod request {
    use crate::internal::message::{InternalForwardMessage, InternalSegment};

    use super::response::*;
    use std::num::ParseIntError;

    use ioevent::rpc::*;
    use serde::{Deserialize, Serialize};
    use sithra_common::model::{Channel, MessageId, UserId};

    pub trait OneBotRequest {
        type RESPONSE;
    }

    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct NoneRequest;
    impl OneBotRequest for NoneRequest {
        type RESPONSE = ();
    }
    /// API请求包装结构
    ///
    /// # 字段说明
    /// - `echo`: 请求标识符，用于匹配异步响应
    /// - `kind`: 具体的API请求类型
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiRequest {
        pub echo: String,
        #[serde(flatten)]
        pub kind: ApiRequestKind,
    }
    impl ApiRequest {
        pub fn new(echo: String, kind: ApiRequestKind) -> Self {
            Self { echo, kind }
        }
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(tag = "action", content = "params")]
    pub enum ApiRequestKind {
        #[serde(rename = "send_private_msg")]
        SendPrivateMsg(SendPrivateMsgParams),
        #[serde(rename = "send_group_msg")]
        SendGroupMsg(SendGroupMsgParams),
        #[serde(rename = "delete_msg")]
        DeleteMsg(DeleteMsgParams),
        #[serde(rename = "get_msg")]
        GetMsg(GetMsgParams),
        #[serde(rename = "set_group_kick")]
        SetGroupKick(SetGroupKickParams),
        #[serde(rename = "set_group_ban")]
        SetGroupBan(SetGroupBanParams),
        #[serde(rename = "set_group_admin")]
        SetGroupAdmin(SetGroupAdminParams),
        #[serde(rename = "set_group_card")]
        SetGroupCard(SetGroupCardParams),
        #[serde(rename = "set_group_leave")]
        SetGroupLeave(SetGroupLeaveParams),
        #[serde(rename = "set_friend_add_request")]
        SetFriendAddRequest(SetFriendAddRequestParams),
        #[serde(rename = "set_group_add_request")]
        SetGroupAddRequest(SetGroupAddRequestParams),
        #[serde(rename = "get_stranger_info")]
        GetStrangerInfo(GetStrangerInfoParams),
        #[serde(rename = "get_group_info")]
        GetGroupInfo(GetGroupInfoParams),
        #[serde(rename = "get_group_member_info")]
        GetGroupMemberInfo(GetGroupMemberInfoParams),
        #[serde(rename = "get_group_member_list")]
        GetGroupMemberList(GetGroupMemberListParams),
        #[serde(rename = "send_forward_msg")]
        CreateForwardMsg(CreateForwardMsgParams),
    }

    /// 发送私聊消息参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SendPrivateMsgParams {
        user_id: String,
        message: Vec<InternalSegment>,
        auto_escape: bool,
    }
    impl OneBotRequest for SendPrivateMsgParams {
        type RESPONSE = MessageIdResponse;
    }
    impl SendPrivateMsgParams {
        pub fn new(user_id: UserId, message: Vec<InternalSegment>) -> Self {
            Self {
                user_id: user_id.to_string(),
                message,
                auto_escape: false,
            }
        }
    }

    /// 发送群消息参数（结构同私聊）
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SendGroupMsgParams {
        group_id: String,
        message: Vec<InternalSegment>,
        auto_escape: bool,
    }
    impl OneBotRequest for SendGroupMsgParams {
        type RESPONSE = MessageIdResponse;
    }
    impl SendGroupMsgParams {
        /// 创建发送群消息参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `message`: 消息内容
        pub fn new(group_id: Channel, message: Vec<InternalSegment>) -> Self {
            Self {
                group_id: group_id.id().to_string(),
                message,
                auto_escape: false,
            }
        }
    }

    /// 消息撤回参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct DeleteMsgParams {
        message_id: String,
    }
    impl OneBotRequest for DeleteMsgParams {
        type RESPONSE = ();
    }
    impl DeleteMsgParams {
        /// 创建消息撤回参数
        ///
        /// # 参数
        /// - `message_id`: 要撤回的消息ID
        pub fn new(message_id: MessageId) -> Self {
            Self {
                message_id: message_id.to_string(),
            }
        }
    }

    /// 获取消息参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetMsgParams {
        message_id: String,
    }
    impl OneBotRequest for GetMsgParams {
        type RESPONSE = MessageDetail;
    }
    impl GetMsgParams {
        /// 创建获取消息参数
        ///
        /// # 参数
        /// - `message_id`: 目标消息ID
        pub fn new(message_id: MessageId) -> Self {
            Self {
                message_id: message_id.to_string(),
            }
        }
    }

    /// 群组踢人参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupKickParams {
        group_id: String,
        user_id: String,
        reject_add_request: bool,
    }
    impl OneBotRequest for SetGroupKickParams {
        type RESPONSE = ();
    }
    impl SetGroupKickParams {
        /// 创建群组踢人参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 被踢用户QQ号
        /// - `reject_add_request`: 是否拒绝后续加群
        pub fn new(
            group_id: Channel,
            user_id: UserId,
            reject_add_request: bool,
        ) -> Self {
            Self {
                group_id: group_id.id().to_string(),
                user_id: user_id.to_string(),
                reject_add_request,
            }
        }
    }

    /// 群组禁言参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupBanParams {
        group_id: String,
        user_id: String,
        duration: i32,
    }
    impl OneBotRequest for SetGroupBanParams {
        type RESPONSE = ();
    }
    impl SetGroupBanParams {
        /// 创建群组禁言参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 被禁言用户QQ号
        /// - `duration`: 禁言时长（秒）
        pub fn new(
            group_id: Channel,
            user_id: UserId,
            duration: i32,
        ) -> Self {
            Self {
                group_id: group_id.id().to_string(),
                user_id: user_id.to_string(),
                duration,
            }
        }
    }

    /// 设置管理员参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupAdminParams {
        group_id: String,
        user_id: String,
        enable: bool,
    }
    impl OneBotRequest for SetGroupAdminParams {
        type RESPONSE = ();
    }
    impl SetGroupAdminParams {
        /// 创建设置管理员参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 用户QQ号
        /// - `enable`: 是否设置为管理员
        pub fn new(
            group_id: Channel,
            user_id: UserId,
            enable: bool,
        ) -> Self {
            Self {
                group_id: group_id.id().to_string(),
                user_id: user_id.to_string(),
                enable,
            }
        }
    }

    /// 群名片设置参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupCardParams {
        group_id: String,
        user_id: String,
        card: String,
    }
    impl OneBotRequest for SetGroupCardParams {
        type RESPONSE = ();
    }
    impl SetGroupCardParams {
        /// 创建群名片设置参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 用户QQ号
        /// - `card`: 新群名片
        pub fn new(
            group_id: Channel,
            user_id: UserId,
            card: String,
        ) -> Self {
            Self {
                group_id: group_id.id().to_string(),
                user_id: user_id.to_string(),
                card,
            }
        }
    }

    /// 退群参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupLeaveParams {
        group_id: String,
        is_dismiss: bool,
    }
    impl OneBotRequest for SetGroupLeaveParams {
        type RESPONSE = ();
    }
    impl SetGroupLeaveParams {
        /// 创建退群参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `is_dismiss`: 是否解散群
        pub fn new(group_id: Channel, is_dismiss: bool) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.id().to_string(),
                is_dismiss,
            })
        }
    }

    /// 好友请求处理参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetFriendAddRequestParams {
        flag: String,
        approve: bool,
        remark: String,
    }
    impl OneBotRequest for SetFriendAddRequestParams {
        type RESPONSE = ();
    }
    impl SetFriendAddRequestParams {
        /// 创建好友请求处理参数
        ///
        /// # 参数
        /// - `flag`: 请求标识
        /// - `approve`: 是否同意
        /// - `remark`: 备注信息
        pub fn new(flag: String, approve: bool, remark: String) -> Self {
            Self {
                flag,
                approve,
                remark,
            }
        }
    }

    /// 加群请求处理参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct SetGroupAddRequestParams {
        pub flag: String,
        pub sub_type: String,
        pub approve: bool,
        pub reason: String,
    }
    impl OneBotRequest for SetGroupAddRequestParams {
        type RESPONSE = ();
    }
    impl SetGroupAddRequestParams {
        /// 创建加群请求处理参数
        ///
        /// # 参数
        /// - `flag`: 请求标识
        /// - `sub_type`: 请求类型
        /// - `approve`: 是否同意
        /// - `reason`: 拒绝理由
        pub fn new(
            flag: String,
            sub_type: String,
            approve: bool,
            reason: String,
        ) -> Self {
            Self {
                flag,
                sub_type,
                approve,
                reason,
            }
        }
    }

    /// 陌生人信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetStrangerInfoParams {
        user_id: i64,
        no_cache: bool,
    }
    impl OneBotRequest for GetStrangerInfoParams {
        type RESPONSE = StrangerInfo;
    }
    impl GetStrangerInfoParams {
        /// 创建陌生人信息查询参数
        ///
        /// # 参数
        /// - `user_id`: 目标QQ号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(user_id: u64, no_cache: bool) -> Self {
            Self {
                user_id: user_id as i64,
                no_cache,
            }
        }
    }

    /// 群信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupInfoParams {
        group_id: String,
        no_cache: bool,
    }
    impl OneBotRequest for GetGroupInfoParams {
        type RESPONSE = GroupInfo;
    }
    impl GetGroupInfoParams {
        /// 创建群信息查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(group_id: Channel, no_cache: bool) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.id().to_string(),
                no_cache,
            })
        }
    }

    /// 群成员信息查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupMemberInfoParams {
        group_id: String,
        user_id: String,
        no_cache: bool,
    }
    impl OneBotRequest for GetGroupMemberInfoParams {
        type RESPONSE = GroupMemberInfo;
    }
    impl GetGroupMemberInfoParams {
        /// 创建群成员信息查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        /// - `user_id`: 成员QQ号
        /// - `no_cache`: 是否不使用缓存
        pub fn new(
            group_id: Channel,
            user_id: UserId,
            no_cache: bool,
        ) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.id().to_string(),
                user_id: user_id.to_string(),
                no_cache,
            })
        }
    }

    /// 群成员列表查询参数
    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct GetGroupMemberListParams {
        group_id: String,
    }
    impl OneBotRequest for GetGroupMemberListParams {
        type RESPONSE = GroupMemberList;
    }
    impl GetGroupMemberListParams {
        /// 创建群成员列表查询参数
        ///
        /// # 参数
        /// - `group_id`: 目标群号
        pub fn new(group_id: Channel) -> Result<Self, ParseIntError> {
            Ok(Self {
                group_id: group_id.id().to_string(),
            })
        }
    }

    #[derive(Debug, Serialize, Deserialize, ProcedureCall)]
    pub struct CreateForwardMsgParams {
        messages: Vec<InternalForwardMessage>,
    }
    impl OneBotRequest for CreateForwardMsgParams {
        type RESPONSE = ForwardIdResponse;
    }
    impl CreateForwardMsgParams {
        pub fn new(messages: Vec<InternalForwardMessage>) -> Self {
            Self {
                messages,
            }
        }
    }

    impl From<SendPrivateMsgParams> for ApiRequestKind {
        fn from(value: SendPrivateMsgParams) -> Self {
            Self::SendPrivateMsg(value)
        }
    }

    impl From<SendGroupMsgParams> for ApiRequestKind {
        fn from(value: SendGroupMsgParams) -> Self {
            Self::SendGroupMsg(value)
        }
    }

    impl From<DeleteMsgParams> for ApiRequestKind {
        fn from(value: DeleteMsgParams) -> Self {
            Self::DeleteMsg(value)
        }
    }

    impl From<GetMsgParams> for ApiRequestKind {
        fn from(value: GetMsgParams) -> Self {
            Self::GetMsg(value)
        }
    }

    impl From<SetGroupKickParams> for ApiRequestKind {
        fn from(value: SetGroupKickParams) -> Self {
            Self::SetGroupKick(value)
        }
    }

    impl From<SetGroupBanParams> for ApiRequestKind {
        fn from(value: SetGroupBanParams) -> Self {
            Self::SetGroupBan(value)
        }
    }

    impl From<SetGroupAdminParams> for ApiRequestKind {
        fn from(value: SetGroupAdminParams) -> Self {
            Self::SetGroupAdmin(value)
        }
    }

    impl From<SetGroupCardParams> for ApiRequestKind {
        fn from(value: SetGroupCardParams) -> Self {
            Self::SetGroupCard(value)
        }
    }

    impl From<SetGroupLeaveParams> for ApiRequestKind {
        fn from(value: SetGroupLeaveParams) -> Self {
            Self::SetGroupLeave(value)
        }
    }

    impl From<SetFriendAddRequestParams> for ApiRequestKind {
        fn from(value: SetFriendAddRequestParams) -> Self {
            Self::SetFriendAddRequest(value)
        }
    }

    impl From<SetGroupAddRequestParams> for ApiRequestKind {
        fn from(value: SetGroupAddRequestParams) -> Self {
            Self::SetGroupAddRequest(value)
        }
    }

    impl From<GetStrangerInfoParams> for ApiRequestKind {
        fn from(value: GetStrangerInfoParams) -> Self {
            Self::GetStrangerInfo(value)
        }
    }

    impl From<GetGroupInfoParams> for ApiRequestKind {
        fn from(value: GetGroupInfoParams) -> Self {
            Self::GetGroupInfo(value)
        }
    }

    impl From<GetGroupMemberInfoParams> for ApiRequestKind {
        fn from(value: GetGroupMemberInfoParams) -> Self {
            Self::GetGroupMemberInfo(value)
        }
    }

    impl From<GetGroupMemberListParams> for ApiRequestKind {
        fn from(value: GetGroupMemberListParams) -> Self {
            Self::GetGroupMemberList(value)
        }
    }

    impl From<CreateForwardMsgParams> for ApiRequestKind {
        fn from(value: CreateForwardMsgParams) -> Self {
            Self::CreateForwardMsg(value)
        }
    }
}