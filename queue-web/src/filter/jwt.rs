use uuid::Uuid;
use warp::Filter;

use crate::filter;
use crate::model::user::{AuthInfo, Role};
use crate::Application;
use crate::{reject, reject_result};
use chrono::Utc;

async fn auth(
    header: Option<String>,
    app: &Application,
    roles: Vec<Role>,
) -> Result<AuthInfo, warp::Rejection> {
    if let Some(bearer_string) = header {
        let header: Vec<String> = bearer_string
            .split_ascii_whitespace()
            .map(String::from)
            .collect();
        if header.len() != 2 || header[0] != "Bearer" {
            reject!("Token format error");
        }
        let access_token = reject_result!(app.jwt.decode_token(&header[1]));
        if Utc::now().timestamp() >= access_token.exp {
            reject!("This session has been expired", 401);
        }
        if !roles.is_empty() && !roles.contains(&access_token.role) {
            reject!("Forbidden", 403)
        }
        Ok(AuthInfo::from(access_token))
    } else {
        reject!("Not auth", 401);
    }
}

pub fn jwt_filter(
    app: &'static Application,
    roles: Vec<Role>,
) -> impl Filter<Extract = (AuthInfo,), Error = warp::Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and(filter::with_app(app))
        .and(warp::any().map(move || roles.clone()))
        .and_then(auth)
}

pub fn refresh_filter() -> impl Filter<Extract = (Uuid,), Error = warp::Rejection> + Clone {
    warp::filters::cookie::optional("refreshToken").and_then(|token: Option<String>| async move {
        if let Some(token) = token {
            Ok(reject_result!(
                Uuid::parse_str(&token).map_err(|error| anyhow::anyhow!("{:?}", error))
            ))
        } else {
            reject!("Not auth", 401);
        }
    })
}
