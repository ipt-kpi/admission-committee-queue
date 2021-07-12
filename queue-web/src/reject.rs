use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::{Rejection, Reply};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Error {
    message: String,
    status: u16,
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Error {
            message: message.to_string(),
            status: 400,
        }
    }
}

impl From<(&str, u16)> for Error {
    fn from(reject: (&str, u16)) -> Self {
        Error {
            message: reject.0.to_string(),
            status: reject.1,
        }
    }
}

impl From<(anyhow::Error, u16)> for Error {
    fn from(reject: (anyhow::Error, u16)) -> Self {
        Error {
            message: format!("{}", reject.0),
            status: reject.1,
        }
    }
}

impl From<anyhow::Error> for Error {
    fn from(error: anyhow::Error) -> Self {
        Error {
            message: format!("{}", error),
            status: 400,
        }
    }
}

impl Reject for Error {}

pub async fn recover(error: Rejection) -> Result<impl Reply, Infallible> {
    let error = match error.find::<Error>() {
        Some(error) => error.clone(),
        None => Error {
            message: format!("{:?}", error),
            status: 400,
        },
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&error),
        StatusCode::from_u16(error.status).unwrap_or(StatusCode::BAD_REQUEST),
    ))
}

#[macro_export]
macro_rules! reject {
    ($error:expr) => {
        return Err(warp::reject::custom(crate::reject::Error::from($error)));
    };
    ($error:expr, $status:expr) => {
        return Err(warp::reject::custom(crate::reject::Error::from((
            $error, $status,
        ))));
    };
}

#[macro_export]
macro_rules! reject_result {
    ($result:expr) => {
        match $result {
            Ok(object) => object,
            Err(error) => reject!(error),
        }
    };
    ($result:expr, $status:expr) => {
        match $result {
            Ok(object) => object,
            Err(error) => reject!(error, $status),
        }
    };
}

#[macro_export]
macro_rules! reject_if_negative {
    ($statement:expr, $error:expr) => {
        if !reject_result!($statement) {
            reject!($error);
        }
    };
    ($statement:expr, $error:expr, $status:expr) => {
        if !reject_result!($statement) {
            reject!($error, $status);
        }
    };
}

#[macro_export]
macro_rules! reject_if {
    ($statement:expr, $error:expr) => {
        if reject_result!($statement) {
            reject!($error);
        }
    };
    ($statement:expr, $error:expr, $status:expr) => {
        if reject_result!($statement) {
            reject!($error, $status);
        }
    };
}
