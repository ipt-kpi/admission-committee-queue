use chrono::{Local, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use teloxide::prelude::*;

use crate::database::Database;
use crate::dialogue::states::{ReceiveDayState, ReceiveIntervalState};
use crate::dialogue::Dialogue;
use crate::queue::Queue;

#[derive(Clone, Serialize, Deserialize)]
pub struct ReceiveTimeState {
    pub date: NaiveDate,
    pub first_time: NaiveTime,
    pub second_time: NaiveTime,
}

#[teloxide(subtransition)]
async fn receive_day(
    state: ReceiveTimeState,
    cx: TransitionIn<AutoSend<Bot>>,
    ans: String,
) -> TransitionOut<Dialogue> {
    match ans.as_str() {
        "Повернутись назад 🔙" => {
            let date = state.date;
            match Queue::global().get_intervals_keyboard(date).await {
                Ok(keyboard) => {
                    cx.answer("Виберіть проміжок часу")
                        .reply_markup(keyboard)
                        .await?;
                    next(Dialogue::ReceiveInterval(ReceiveIntervalState { date }))
                }
                Err(error) => {
                    cx.answer(error.to_string()).await?;
                    next(Dialogue::ReceiveTime(state))
                }
            }
        }
        "Вибір іншої дати 🔙" => {
            cx.answer("Виберіть день тижня для запису")
                .reply_markup(Queue::global().get_days_keyboard())
                .await?;
            next(Dialogue::ReceiveDay(ReceiveDayState))
        }
        time => match NaiveTime::parse_from_str(time, "%H:%M") {
            Ok(time) => {
                let date = state.date;
                let current_date = Local::now().date().naive_utc();
                if current_date > date {
                    cx.answer("Ви не можете більше записатися на цей день, виберіть інше число")
                        .reply_markup(Queue::global().get_days_keyboard())
                        .await?;
                    return next(Dialogue::ReceiveDay(ReceiveDayState));
                }
                let database = Database::global();
                match database.check_time(date, time).await {
                    Ok(exists) => {
                        if exists {
                            match Queue::global()
                                .get_relevant_time_keyboard(
                                    date,
                                    state.first_time,
                                    state.second_time,
                                )
                                .await
                            {
                                Ok(keyboard) => {
                                    cx.answer(
                                        "Не вдалося записатися на даний час, його вже зайнято",
                                    )
                                    .reply_markup(keyboard)
                                    .await?;
                                }
                                Err(error) => {
                                    cx.answer(error.to_string()).await?;
                                }
                            }
                            next(Dialogue::ReceiveTime(state))
                        } else {
                            match database
                                .register_in_queue(cx.update.chat_id(), date, time)
                                .await
                            {
                                Ok(old_record) => {
                                    match Queue::global()
                                        .get_relevant_time_keyboard(
                                            date,
                                            state.first_time,
                                            state.second_time,
                                        )
                                        .await
                                    {
                                        Ok(keyboard) => {
                                            let msg = if old_record {
                                                cx.answer(format!("Ви були зареєстровані в черзі на новий час: {} {} (старий запис не актуальний)", date, time))
                                            } else {
                                                cx.answer(format!(
                                                    "Ви були зареєстровані в черзі на: {} {}\nЯкщо бажаєте завжди слідкувати за чергою то введіть /toggle_notification (це ж саме й для вимкнення)",
                                                    date, time
                                                ))
                                            };
                                            msg.reply_markup(keyboard).await?;
                                        }
                                        Err(error) => {
                                            cx.answer(error.to_string()).await?;
                                        }
                                    }
                                    next(Dialogue::ReceiveTime(state))
                                }
                                Err(error) => {
                                    cx.answer(
                                        "Не вдалося зареєструватись в черзі, виникла помилка",
                                    )
                                    .await?;
                                    log::error!("Database error: {}", error);
                                    next(Dialogue::ReceiveTime(state))
                                }
                            }
                        }
                    }
                    Err(error) => {
                        cx.answer("Не вдалося перевірити чи зайнятий обраний час")
                            .await?;
                        log::error!("Database error: {}", error);
                        next(Dialogue::ReceiveTime(state))
                    }
                }
            }
            Err(_) => {
                cx.answer("Введено невірний формат часу").await?;
                next(Dialogue::ReceiveTime(state))
            }
        },
    }
}
