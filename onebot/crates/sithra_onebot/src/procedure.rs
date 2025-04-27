use ioevent::rpc::*;
use sithra_common::{adapt_api, api::*, model::*};
use sithra_onebot_common::message::OneBotMessage;

use crate::{
    OneBotGenericId,
    internal::{api::request, message::InternalSegment},
    state::OneBotAdapterState,
};

#[adapt_api(OneBotGenericId)]
pub async fn send_message(
    state: State<OneBotAdapterState>,
    data: SithraCall<SendMessage>,
) -> Result {
    let message: SVec<InternalSegment> = data
        .message::<OneBotMessage>()
        .into_iter()
        .map(|s| s.into())
        .collect();
    let echo = state.next_echo().await;
    let channel = data.take_call().channel;
    match channel.channel_type() {
        ChannelType::Private => {
            let request = request::SendPrivateMsgParams::new(channel, message);
            let response = state.api_client.call_api(echo, request).await?;
            let response = SendMessageResponse {
                message_id: Some(MessageId::new(response.message_id)),
            };
            Ok(response)
        }
        ChannelType::Group => {
            let request = request::SendGroupMsgParams::new(channel, message);
            let response = state.api_client.call_api(echo, request).await?;
            let response = SendMessageResponse {
                message_id: Some(MessageId::new(response.message_id)),
            };
            Ok(response)
        }
    }
}
