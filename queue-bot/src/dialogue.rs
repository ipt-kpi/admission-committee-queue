use teloxide::macros::Transition;

use crate::dialogue::states::{ReceiveCaptchaState, ReceiveFullNameState};
use crate::dialogue::states::{ReceivePhoneState, StartState};

mod states;

#[derive(Transition)]
pub enum Dialogue {
    Start(StartState),
    ReceiveCaptcha(ReceiveCaptchaState),
    ReceiveFullName(ReceiveFullNameState),
    ReceivePhone(ReceivePhoneState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}
