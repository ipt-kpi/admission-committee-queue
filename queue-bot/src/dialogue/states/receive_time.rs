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
        "Назад 🔙" => {
            let date = state.date;
            match Queue::global().get_intervals_keyboard(date).await {
                Ok(keyboard) => {
                    cx.answer("Выберите промежуток времени")
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
        "Выбор другой даты 🔙" => {
            cx.answer("Выберите день недели для записи")
                .reply_markup(Queue::global().get_days_keyboard())
                .await?;
            next(Dialogue::ReceiveDay(ReceiveDayState))
        }
        time => {
            match NaiveTime::parse_from_str(time, "%H:%M") {
                Ok(time) => {
                    let date = state.date;
                    let current_date = Local::now().date().naive_utc();
                    if current_date > date {
                        cx.answer(
                            "Вы не можете больше записаться на данный день, выберите другое число",
                        )
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
                                            "Не удалось записаться на данное время, его уже заняли",
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
                                        if old_record {
                                            cx.answer(format!("Вы были зарегистрированы в очереди на новое время: {} {} (старая запись не актуальна)", date, time)).await?;
                                        } else {
                                            cx.answer(format!(
                                                "Вы были зарегистрированы в очереди на: {} {}",
                                                date, time
                                            ))
                                            .await?;
                                        }
                                        //TODO new state
                                        next(Dialogue::ReceiveTime(state))
                                    }
                                    Err(error) => {
                                        cx.answer("Не удалось зарегистрироватся в очереди, возникла ошибка").await?;
                                        log::error!("Database error: {}", error);
                                        next(Dialogue::ReceiveTime(state))
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            cx.answer("Не удалось проверить занято ли выбранное время")
                                .await?;
                            log::error!("Database error: {}", error);
                            next(Dialogue::ReceiveTime(state))
                        }
                    }
                }
                Err(_) => {
                    cx.answer("Введен не верный формат времени").await?;
                    next(Dialogue::ReceiveTime(state))
                }
            }
        }
    }
}
