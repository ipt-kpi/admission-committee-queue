use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::dialogue::states::{ReceiveDayState, ReceiveTimeState};
use crate::dialogue::Dialogue;
use crate::queue::Queue;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveIntervalState {
    pub date: NaiveDate,
}

#[teloxide(subtransition)]
async fn receive_interval(
    state: ReceiveIntervalState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    if ans == "Повернутись назад 🔙" {
        cx.answer("Виберіть день тижня")
            .reply_markup(Queue::global().get_days_keyboard())
            .await?;
        next(Dialogue::ReceiveDay(ReceiveDayState))
    } else {
        match parse_interval(ans) {
            Some((first_time, second_time)) => {
                let date = state.date;
                match Queue::global()
                    .get_relevant_time_keyboard(date, first_time, second_time)
                    .await
                {
                    Ok(keyboard) => {
                        cx.answer("Виберіть час")
                            .reply_markup(keyboard)
                            .send()
                            .await?;
                        next(Dialogue::ReceiveTime(ReceiveTimeState {
                            date,
                            first_time,
                            second_time,
                        }))
                    }
                    Err(error) => {
                        cx.answer(error.to_string()).await?;
                        next(Dialogue::ReceiveInterval(state))
                    }
                }
            }
            None => {
                cx.answer("Введено невірний формат часу").await?;
                next(Dialogue::ReceiveInterval(state))
            }
        }
    }
}

fn parse_interval(interval: String) -> Option<(NaiveTime, NaiveTime)> {
    let mut interval = interval
        .split("-")
        .filter_map(|time| NaiveTime::parse_from_str(time, "%H:%M").ok());
    interval
        .next()
        .map(|first_time| interval.next().map(|second_time| (first_time, second_time)))
        .unwrap_or(None)
}
