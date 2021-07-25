use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dialogue::Dialogue;

#[derive(Clone, Serialize, Deserialize)]
pub struct BannedState;

#[teloxide(subtransition)]
async fn banned(
    state: BannedState,
    cx: TransitionIn<AutoSend<Bot>>,
    _ans: String,
) -> TransitionOut<Dialogue> {
    cx.answer("Ви були заблоковані, якщо вважаєте, що виникла помилка то зверніться до оператора технічної підтримки").await?;
    next(Dialogue::Banned(state))
}
