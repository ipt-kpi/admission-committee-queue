use once_cell::sync::OnceCell;
use regex::Regex;
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::database::Database;
use crate::dialogue::states::{ReceiveDayState, ReceiveFullNameState};
use crate::dialogue::Dialogue;
use crate::queue::Queue;
use crate::user::Enrollee;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceivePhoneState {
    name: String,
    patronymic: String,
    last_name: String,
}

impl ReceivePhoneState {
    pub fn new(name: String, patronymic: String, last_name: String) -> Self {
        ReceivePhoneState {
            name,
            patronymic,
            last_name,
        }
    }
}

static PHONE_REGEX: OnceCell<Regex> = OnceCell::new();

#[teloxide(subtransition)]
async fn receive_phone(
    state: ReceivePhoneState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    let regex = PHONE_REGEX.get_or_init(|| {
        Regex::new(r"^\+?3?8?(0\d{9})$").expect("Failed to create phone number regex!")
    });
    if regex.is_match(&ans) {
        cx.answer(format!(
            "Підсумкові дані: \n\
             Прізвище: {} \n\
             Ім'я: {} \n\
             По батькові: {} \n\
             Телефон: {}",
            state.last_name, state.name, state.patronymic, ans
        ))
        .await?;

        match cx.update.from() {
            Some(user) => {
                let enrollee = Enrollee {
                    chat_id: cx.update.chat.id,
                    username: user.username.as_ref().map_or(String::new(), String::from),
                    name: state.name,
                    patronymic: state.patronymic,
                    last_name: state.last_name,
                    phone_number: ans,
                };
                if let Err(error) = Database::global().register(enrollee).await {
                    log::error!("Database error while register: {}", error);
                    cx.answer("Помилка при реєстрації користувача!").await?;
                    next(Dialogue::ReceiveFullName(ReceiveFullNameState))
                } else {
                    cx.answer("Виберіть день тижня для запису")
                        .reply_markup(Queue::global().get_days_keyboard())
                        .await?;
                    next(Dialogue::ReceiveDay(ReceiveDayState))
                }
            }
            None => {
                cx.answer("Не вдалося отримати дані про користувача, спробуйте ще раз ввести ПІБ")
                    .await?;
                next(Dialogue::ReceiveFullName(ReceiveFullNameState))
            }
        }
    } else {
        cx.answer("Неправильно введено номер телефону, спробуйте ще раз!")
            .await?;
        next(Dialogue::ReceivePhone(state))
    }
}
