use anyhow::{Context, Result};
use chrono::{Local, NaiveDate, NaiveTime};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use teloxide::types::{KeyboardButton, KeyboardMarkup};

use crate::config;
use crate::database::Database;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schedule {
    #[serde(with = "config::time_format")]
    pub start_time: NaiveTime,
    pub interval: u16,
    pub max_enrollee: u16,
}

static INSTANCE: OnceCell<Queue> = OnceCell::new();

pub struct Queue {
    agree_keyboard: KeyboardMarkup,
    schedule: HashMap<NaiveDate, Schedule>,
}

pub async fn initialize(schedule: HashMap<NaiveDate, Schedule>) -> Result<()> {
    let agree_keyboard = KeyboardMarkup::default()
        .append_row(vec![KeyboardButton::new("✅"), KeyboardButton::new("❌")])
        .resize_keyboard(true);
    let queue = Queue {
        agree_keyboard,
        schedule,
    };
    INSTANCE
        .set(queue)
        .map_err(|_| anyhow::anyhow!("Failed to initialize database!"))
}

impl Queue {
    pub fn global() -> &'static Queue {
        INSTANCE.get().expect("Pool isn't initialized")
    }

    pub fn get_agree_keyboard(&self) -> KeyboardMarkup {
        self.agree_keyboard.clone()
    }

    pub fn get_days_keyboard(&self) -> KeyboardMarkup {
        let current_date = Local::now().date().naive_utc();
        Self::gen_two_columns_keyboard(
            self.schedule
                .keys()
                .filter(|&date| date >= &current_date)
                .map(|date| date.format("%d.%m").to_string()),
        )
    }

    pub async fn get_intervals_keyboard(&self, date: NaiveDate) -> Result<KeyboardMarkup> {
        let schedule = self
            .schedule
            .get(&date)
            .context("Указанный день не найден")?;
        match Database::global()
            .get_intervals(
                date,
                schedule.start_time,
                schedule.max_enrollee,
                schedule.interval,
            )
            .await
        {
            Ok(intervals) => {
                let keyboard = Self::gen_two_columns_keyboard(intervals.into_iter());
                Ok(keyboard.append_row(vec![KeyboardButton::new("Назад 🔙")]))
            }
            Err(error) => {
                log::error!("Database error: {}", error);
                Err(anyhow::anyhow!("Произошла ошибка при выполнение команды"))
            }
        }
    }

    pub async fn get_relevant_time_keyboard(
        &self,
        date: NaiveDate,
        first_time: NaiveTime,
        second_time: NaiveTime,
    ) -> Result<KeyboardMarkup> {
        let schedule = self
            .schedule
            .get(&date)
            .context("Указанный день не найден")?;
        match Database::global()
            .get_intervals_between(
                date,
                schedule.start_time,
                schedule.max_enrollee,
                schedule.interval,
                first_time,
                second_time,
            )
            .await
        {
            Ok(intervals) => {
                let keyboard = Self::gen_two_columns_keyboard(intervals.into_iter());
                Ok(keyboard.append_row(vec![
                    KeyboardButton::new("Назад 🔙"),
                    KeyboardButton::new("Выбор другой даты 🔙"),
                ]))
            }
            Err(error) => {
                log::error!("Database error: {}", error);
                Err(anyhow::anyhow!("Произошла ошибка при выполнение команды"))
            }
        }
    }

    fn gen_two_columns_keyboard(buttons: impl Iterator<Item = String>) -> KeyboardMarkup {
        let keyboard = buttons
            .map(KeyboardButton::new)
            .collect::<Vec<_>>()
            .chunks(2)
            .map(|x| x.to_vec())
            .collect::<Vec<_>>();
        KeyboardMarkup::new(keyboard).resize_keyboard(true)
    }
}
