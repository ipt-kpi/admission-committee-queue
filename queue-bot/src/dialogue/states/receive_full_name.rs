use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::database::Database;
use crate::dialogue::states::ReceivePhoneState;
use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveFullNameState;

#[teloxide(subtransition)]
async fn receive_full_name(
    state: ReceiveFullNameState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    let mut full_name = ans.split_whitespace();
    if full_name.clone().by_ref().count() != 3usize {
        cx.answer("Неправильно введено ПІБ, спробуйте ще раз!")
            .await?;
        next(Dialogue::ReceiveFullName(state))
    } else {
        let last_name = full_name.next().unwrap();
        let name = full_name.next().unwrap();
        let patronymic = full_name.next().unwrap();
        match Database::global()
            .is_enrollee_valid(last_name, name, patronymic)
            .await
        {
            Ok(success) => {
                if success {
                    let receive_phone_state = ReceivePhoneState::new(
                        name.to_string(),
                        patronymic.to_string(),
                        last_name.to_string(),
                    );
                    cx.answer("Введіть номер телефону в форматі +380XXXXXXXXX або 0XXXXXXXXX")
                        .await?;
                    next(Dialogue::ReceivePhone(receive_phone_state))
                } else {
                    cx.answer("Вас не вдалося знайти в списку заявок на вступ, можливо ви помилитися при введенні даних. Спробуйте ще раз.").await?;
                    next(Dialogue::ReceiveFullName(state))
                }
            }
            Err(error) => {
                cx.answer(
                    "Помилка під час перевірки валідності користувача, спробуйте ще раз ввести ПІБ",
                )
                .await?;
                log::error!("Database error while enrollee check: {}", error);
                next(Dialogue::ReceiveFullName(state))
            }
        }
    }
}
