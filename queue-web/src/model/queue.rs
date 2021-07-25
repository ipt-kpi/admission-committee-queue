use chrono::{NaiveDate, NaiveTime};
use hyper::StatusCode;
use sqlx::FromRow;
use std::fmt;
use warp::http::Response;
use warp::Reply;

use crate::model::enrollee::Status;

#[derive(FromRow)]
pub struct Queue {
    pub last_name: String,
    pub name: String,
    pub patronymic: String,
    pub date: NaiveDate,
    pub time: NaiveTime,
    pub phone_number: String,
    pub username: String,
    pub status: Status,
    pub id: i64,
}

pub struct StudentsQueue(pub Vec<Queue>);

impl fmt::Display for Queue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{},{},{}",
            self.last_name,
            self.name,
            self.patronymic,
            self.date,
            self.time,
            self.phone_number,
            self.username,
            self.status,
            self.id
        )
    }
}

impl fmt::Display for StudentsQueue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.iter().fold(Ok(()), |result, lesson| {
            result.and_then(|_| writeln!(f, "{}", lesson))
        })
    }
}

impl Reply for StudentsQueue {
    fn into_response(self) -> warp::reply::Response {
        let body = format!(
            "last_name,name,patronymic,date,time,phone_number,username,status,id\n{}",
            self
        )
        .into();
        Response::builder()
            .header("Content-Type", "text/csv")
            .header("Content-Disposition", "attachment;filename=queue.csv")
            .body(body)
            .unwrap_or(
                warp::reply::with_status(
                    "Failed to create dump",
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response(),
            )
    }
}
