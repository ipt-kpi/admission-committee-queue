use teloxide::prelude::*;

use crate::dialogue::states::StartState;
use crate::dialogue::Dialogue;
use regex::Regex;
use once_cell::sync::OnceCell;

#[derive(Clone)]
pub struct ReceivePhoneState {
    name: String,
    patronymic: String,
    surname: String,
}

impl ReceivePhoneState {
    pub fn new(name: String, patronymic: String, surname: String) -> Self {
        ReceivePhoneState {
            name,
            patronymic,
            surname,
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
            Имя: {}\n\
            Отчество: {}\n\
            Фамилия: {}\n\
            Телефон: {}",
            state.name, state.patronymic, state.surname, ans
        ))
        .await?;
        next(Dialogue::Start(StartState))
    } else {
        cx.answer("Неверно введен номер телефона, попробуйте еще раз!")
            .await?;
        next(Dialogue::ReceivePhone(state))
    }
}
