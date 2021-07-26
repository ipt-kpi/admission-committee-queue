use anyhow::Result;
use chrono::{Duration, TimeZone, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;
use warp::Reply;

use crate::model::user::{AuthInfo, Role};
use crate::{reject, reject_result};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessToken {
    pub sub: i32,
    pub exp: i64,
    pub username: String,
    pub role: Role,
}

pub struct Jwt {
    decoding_key: DecodingKey<'static>,
    encoding_key: EncodingKey,
}

impl From<AuthInfo> for AccessToken {
    fn from(auth_info: AuthInfo) -> Self {
        AccessToken {
            sub: auth_info.id,
            exp: (Utc::now() + Duration::minutes(15)).timestamp(),
            username: auth_info.username,
            role: auth_info.role,
        }
    }
}

impl Jwt {
    pub fn new<P: AsRef<Path>>(public_key: P, private_key: P) -> Result<Self> {
        let mut decoding_buffer = Box::leak(Box::new(Vec::new()));
        let mut encoding_buffer = Box::leak(Box::new(Vec::new()));
        File::open(public_key)?.read_to_end(&mut decoding_buffer)?;
        File::open(private_key)?.read_to_end(&mut encoding_buffer)?;
        Ok(Jwt {
            decoding_key: DecodingKey::from_ec_pem(decoding_buffer)?,
            encoding_key: EncodingKey::from_ec_pem(encoding_buffer)?,
        })
    }

    pub fn create_token(&self, access_token: &AccessToken) -> Result<String> {
        Ok(jsonwebtoken::encode(
            &Header::new(Algorithm::ES256),
            access_token,
            &self.encoding_key,
        )?)
    }

    pub fn decode_token(&self, token: &str) -> Result<AccessToken> {
        Ok(jsonwebtoken::decode(
            token,
            &self.decoding_key,
            &Validation::new(Algorithm::ES256),
        )?
        .claims)
    }

    pub fn create_session_reply(
        &self,
        refresh_session: (Uuid, i64),
        auth_info: AuthInfo,
    ) -> Result<impl Reply, warp::Rejection> {
        let access_token = AccessToken::from(auth_info);
        let reply = warp::reply::json(&serde_json::json!({
            "accessToken": reject_result!(self.create_token(&access_token)),
            "exp": access_token.exp,
            "refreshSession": refresh_session.1,
            "role": access_token.role
        }));
        let expires = Utc
            .timestamp(refresh_session.1, 0)
            .format("%a, %d %b %Y %H:%M:%S GMT")
            .to_string();
        Ok(warp::reply::with_header(
            reply,
            "Set-Cookie",
            format!(
                "refreshToken={}; Expires={}; Path=/api/user/auth; HttpOnly; SameSite=None; Secure",
                refresh_session.0, expires,
            ),
        ))
    }

    pub fn invalidate_session(&self) -> Result<impl Reply, warp::Rejection> {
        Ok(warp::reply::with_header(
            warp::reply::reply(),
            "Set-Cookie", 
            "refreshToken=invalid; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Path=/api/user/auth; HttpOnly; SameSite=None; Secure"
        ))
    }
}
