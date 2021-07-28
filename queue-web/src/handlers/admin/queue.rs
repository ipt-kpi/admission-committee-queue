use chrono::NaiveDate;
use serde::Deserialize;
use warp::Reply;

use crate::model::enrollee::{Enrollee, Status};
use crate::model::user::AuthInfo;
use crate::Application;
use crate::{reject, reject_result};

pub async fn dates(
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    let dates = reject_result!(app.database.get_dates().await);
    Ok(warp::reply::json(&serde_json::json!({ "dates": dates })))
}

pub async fn enrollees(
    dates: Vec<NaiveDate>,
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    let enrollees = reject_result!(app.database.get_enrollees(dates).await);
    Ok(warp::reply::json(&serde_json::json!({
        "enrollees": enrollees
    })))
}

pub async fn status(
    id: i64,
    status: Status,
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    reject_result!(app.database.change_status(id, status).await);
    Ok(warp::reply::reply())
}

pub async fn update(
    enrollee: Enrollee,
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    reject_result!(app.database.update_enrollee(enrollee).await);
    Ok(warp::reply::reply())
}

pub async fn students_queue(
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    match app.database.get_students_queue().await {
        Ok(queue) => Ok(queue),
        Err(error) => reject!(error),
    }
}

pub async fn relevant_time(
    date: NaiveDate,
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    let relevant_time = reject_result!(app.database.get_relevant_time(date).await);
    Ok(warp::reply::json(&serde_json::json!({
        "relevantTime": relevant_time
    })))
}

#[derive(Deserialize)]
pub struct QueueInfo {
    pub last_name: String,
    pub name: String,
    pub patronymic: String,
    pub phone_number: String,
    pub date: String,
    pub time: String,
}

pub async fn register(
    info: QueueInfo,
    app: &'static Application,
    _auth_info: AuthInfo,
) -> Result<impl Reply, warp::Rejection> {
    let id = reject_result!(app.database.register_in_queue(info).await);
    Ok(warp::reply::json(&serde_json::json!({ "id": id })))
}
