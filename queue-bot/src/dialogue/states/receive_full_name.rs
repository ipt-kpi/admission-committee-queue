use teloxide::prelude::*;

use crate::dialogue::states::{ReceivePhoneState, StartState};
use crate::dialogue::Dialogue;

#[derive(Clone)]
pub struct ReceiveFullNameState;

#[teloxide(subtransition)]
async fn receive_full_name(
    state: ReceiveFullNameState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    let mut full_name = ans.split_whitespace();
    if full_name.clone().by_ref().count() != 3usize {
        cx.answer("Неверно введено ФИО, попробуйте еще раз!")
            .await?;
        next(Dialogue::ReceiveFullName(state))
    } else {
        let name = full_name.next().unwrap();
        let patronymic = full_name.next().unwrap();
        let surname = full_name.next().unwrap();
        let receive_phone_state = ReceivePhoneState::new(
            name.to_string(),
            patronymic.to_string(),
            surname.to_string(),
        );
        cx.answer("Введите номер телефона в формате +380XXXXXXXXX или 0XXXXXXXXX").await?;
        next(Dialogue::ReceivePhone(receive_phone_state))
    }
}
