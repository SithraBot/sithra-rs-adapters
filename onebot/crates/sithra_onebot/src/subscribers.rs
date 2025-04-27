use ioevent::prelude::*;

use crate::procedure::*;
use crate::state::OneBotAdapterState;

pub const SUBSCRIBERS: &[Subscriber<OneBotAdapterState>] = &[create_subscriber!(send_message)];
