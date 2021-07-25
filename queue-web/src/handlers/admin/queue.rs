use chrono::NaiveDate;
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
