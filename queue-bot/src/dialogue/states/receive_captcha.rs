use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::captcha::Captcha;
use crate::dialogue::states::{BannedState, ReceiveFullNameState, StartState};
use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveCaptchaState {
    answer: String,
    attempt_count: u8,
}

pub enum CheckState {
    Correct,
    Incorrect,
    Update,
    Block,
}

impl ReceiveCaptchaState {
    pub fn new(answer: String) -> Self {
        ReceiveCaptchaState {
            answer,
            attempt_count: 0,
        }
    }

    pub fn check_answer(&mut self, answer: String) -> CheckState {
        self.attempt_count = self.attempt_count + 1;
        if self.answer == answer {
            CheckState::Correct
        } else {
            self.check_attempt()
        }
    }

    pub fn change_answer(mut self, answer: String) -> ReceiveCaptchaState {
        self.answer = answer;
        self
    }

    fn check_attempt(&self) -> CheckState {
        if self.attempt_count >= 30 {
            CheckState::Block
        } else if self.attempt_count % 10 == 0 {
            CheckState::Update
        } else {
            CheckState::Incorrect
        }
    }
}

#[teloxide(subtransition)]
async fn receive_captcha(
    mut state: ReceiveCaptchaState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    match state.check_answer(ans) {
        CheckState::Correct => {
            cx.answer("Капча вірна").await?;
            cx.answer("Введіть своє ПІБ через пробіл. Наприклад: 'Іванов Іван Іванович'")
                .await?;
            next(Dialogue::ReceiveFullName(ReceiveFullNameState))
        }
        CheckState::Incorrect => {
            cx.answer("Капча невірна").await?;
            next(Dialogue::ReceiveCaptcha(state))
        }
        CheckState::Update => {
            cx.answer("Занадто велика кількість спроб, генеруємо нову капчу")
                .await?;
            match Captcha::send(&cx).await {
                Ok(answer) => next(Dialogue::ReceiveCaptcha(state.change_answer(answer))),
                Err(error) => {
                    cx.answer("Виникла помилка при створенні капчі")
                        .send()
                        .await?;
                    log::error!("Failed to send captcha: {}", error);
                    next(Dialogue::Start(StartState))
                }
            }
        }
        CheckState::Block => {
            cx.answer("Ви були заблоковані, якщо вважаєте, що виникла помилка то зверніться до оператора технічної підтримки").await?;
            next(Dialogue::Banned(BannedState))
        }
    }
}
