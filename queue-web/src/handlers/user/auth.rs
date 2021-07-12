use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use warp::Reply;

use crate::hash;
use crate::model::user::AuthInfo;
use crate::Application;
use crate::{reject, reject_if, reject_if_negative, reject_result};

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationInfo {
    pub username: String,
    pub password: String,
    pub email: String,
    pub token: String,
}

pub async fn register(
    info: RegistrationInfo,
    app: &'static Application,
) -> Result<impl Reply, warp::Rejection> {
    reject_if_negative!(
        app.recaptcha.check(&info.token).await,
        "Failed to verify captcha"
    );
    reject_if!(
        app.database.user_exists(&info.username).await,
        "This username already in use"
    );
    reject_result!(app.database.create_user(info).await);
    Ok(warp::reply::reply())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginInfo {
    pub username: String,
    pub password: String,
    pub fingerprint: String,
}

pub async fn login(
    info: LoginInfo,
    app: &'static Application,
) -> Result<impl Reply, warp::Rejection> {
    if let Some(user) = reject_result!(app.database.get_user_by_name(&info.username).await) {
        reject_if_negative!(
            hash::verify_password(&info.password, &user.password),
            "Incorrect password"
        );
        let refresh_session = reject_result!(
            app.database
                .create_refresh_session(user.id, &info.fingerprint)
                .await
        );
        app.jwt.create_session_reply(refresh_session, user.into())
    } else {
        reject!("Failed to find account with this username");
    }
}

pub async fn logout(
    app: &'static Application,
    auth_info: AuthInfo,
    refresh_token: Uuid,
) -> Result<impl Reply, warp::Rejection> {
    reject_result!(
        app.database
            .remove_refresh_session(auth_info.id, refresh_token)
            .await
    );
    app.jwt.invalidate_session()
}

pub async fn refresh_session(
    fingerprint: Value,
    app: &'static Application,
    refresh_token: Uuid,
) -> Result<impl Reply, warp::Rejection> {
    let option_fingerprint = reject_result!(fingerprint
        .get("fingerprint")
        .context("Failed to get fingerprint"))
    .as_str();
    let fingerprint =
        reject_result!(option_fingerprint.context("Failed to parse fingerprint as string"));
    let refresh_session = reject_result!(
        app.database
            .update_refresh_session(fingerprint, refresh_token)
            .await,
        401
    );
    if let Some(user) = reject_result!(app.database.get_user_by_id(refresh_session.2).await) {
        app.jwt
            .create_session_reply((refresh_session.0, refresh_session.1), user.into())
    } else {
        reject!("Failed to find account with this id");
    }
}
