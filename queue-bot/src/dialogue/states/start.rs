use teloxide::prelude::*;

use crate::captcha::Captcha;
use crate::dialogue::states::receive_captcha::ReceiveCaptchaState;
use crate::dialogue::Dialogue;

#[derive(Clone)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("Введите капчу").await?;
    match Captcha::send(&cx).await {
        Ok(answer) => next(Dialogue::ReceiveCaptcha(ReceiveCaptchaState::new(answer))),
        Err(_error) => {
            //TODO error
            next(Dialogue::Start(state))
        }
    }
}
