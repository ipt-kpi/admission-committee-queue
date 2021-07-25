use serde::{Deserialize, Serialize};
use teloxide::macros::Transition;

use crate::dialogue::states::{
    BannedState, ReceiveCaptchaState, ReceiveDayState, ReceiveFullNameState, ReceiveIntervalState,
    ReceivePhoneState, ReceiveTimeState, StartState,
};

mod states;

#[derive(Transition, Serialize, Deserialize)]
pub enum Dialogue {
    Banned(BannedState),
    Start(StartState),
    ReceiveCaptcha(ReceiveCaptchaState),
    ReceiveFullName(ReceiveFullNameState),
    ReceivePhone(ReceivePhoneState),
    ReceiveDay(ReceiveDayState),
    ReceiveInterval(ReceiveIntervalState),
    ReceiveTime(ReceiveTimeState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}
