mod config;

use log::info;
use sithra_common::{msg, prelude::*};
use sithra_onebot_common::message::*;

const SUBSCRIBERS: &[ioevent::Subscriber<()>] = &[];

#[sithra_common::main(subscribers = SUBSCRIBERS, state = ())]
async fn main(wright: &ioevent::EffectWright) {
    info!("onebot 适配器启动成功");

    // 这样就可以获取到插件的数据路径
    let _data_path = sithra_common::data_path!();

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
        let event = MessageEvent::new(NonGenericId, channel, user, message);
        let result = wright.emit(&event);
        log::error!("something wrong: {:?}", result);
    }
}
