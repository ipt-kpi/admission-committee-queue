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
            "Итоговые данные: \n\
            Фамилия: {}\n\
            Имя: {}\n\
            Отчество: {}\n\
            Телефон: {}",
            state.last_name, state.name, state.patronymic, ans
        ))
        .await?;

        match cx.update.from() {
            Some(user) => {
                let enrollee = Enrollee {
                    id: cx.update.chat.id,
                    username: user.username.as_ref().map_or(String::new(), String::from),
                    name: state.name,
                    patronymic: state.patronymic,
                    last_name: state.last_name,
                    phone_number: ans,
                };
                if let Err(error) = Database::global().register(enrollee).await {
                    log::error!("Database error while register: {}", error);
                    cx.answer("Произошла ошибка при регистрации пользователя!")
                        .await?;
                    next(Dialogue::ReceiveFullName(ReceiveFullNameState))
                } else {
                    cx.answer("Выберите день недели для записи")
                        .reply_markup(Queue::global().get_days_keyboard())
                        .await?;
                    next(Dialogue::ReceiveDay(ReceiveDayState))
                }
            }
            None => {
                cx.answer(
                    "Не удалось получить данные о пользователе, попробуйте еще раз ввести ФИО",
                )
                .await?;
                next(Dialogue::ReceiveFullName(ReceiveFullNameState))
            }
        }
    } else {
        cx.answer("Неверно введен номер телефона, попробуйте еще раз!")
            .await?;
        next(Dialogue::ReceivePhone(state))
    }
}
