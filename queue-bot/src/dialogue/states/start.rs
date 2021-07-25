use serde::{Deserialize, Serialize};
use teloxide::prelude::*;
use teloxide::types::ReplyMarkup;

use crate::captcha::Captcha;
use crate::dialogue::states::receive_captcha::ReceiveCaptchaState;
use crate::dialogue::Dialogue;
use crate::queue::Queue;

#[derive(Clone, Serialize, Deserialize)]
pub struct StartState;

#[teloxide(subtransition)]
async fn start(
    state: StartState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    if ans == "✅" {
        cx.answer("Введіть капчу")
            .reply_markup(ReplyMarkup::kb_remove())
            .send()
            .await?;
        match Captcha::send(&cx).await {
            Ok(answer) => next(Dialogue::ReceiveCaptcha(ReceiveCaptchaState::new(answer))),
            Err(error) => {
                cx.answer("Відбулася помилки при створенні капчи")
                    .send()
                    .await?;
                log::error!("Failed to send captcha: {}", error);
                next(Dialogue::Start(state))
            }
        }
    } else {
        cx.answer(
            "Щоб продовжити роботу з ботом, погодьтеся зі збором та обробкою персональних даних у вигляді ПІБ та номеру телефону",
        )
        .reply_markup(Queue::global().get_agree_keyboard())
        .send()
        .await?;
        next(Dialogue::Start(state))
    }
}
