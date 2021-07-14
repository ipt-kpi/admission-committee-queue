use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Enrollee {
    pub id: i64,
    pub last_name: String,
    pub name: String,
    pub patronymic: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub processed: bool,
    pub username: String,
    pub phone_number: String,
}
