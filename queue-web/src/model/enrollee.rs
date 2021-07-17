use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use std::str::FromStr;

#[derive(Serialize, Deserialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Enrollee {
    pub id: i64,
    pub last_name: String,
    pub name: String,
    pub patronymic: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub status: Status,
    pub username: String,
    pub phone_number: String,
}

#[derive(Serialize, Deserialize, Type)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Wait,
    Processed,
    Absent,
}

impl FromStr for Status {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "wait" => Ok(Status::Wait),
            "processed" => Ok(Status::Processed),
            "absent" => Ok(Status::Absent),
            _ => Err("Failed to determine status from input"),
        }
    }
}
