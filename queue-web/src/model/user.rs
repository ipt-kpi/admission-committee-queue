use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

use crate::jwt::AccessToken;

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Type)]
#[sqlx(type_name = "role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    User,
}

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

impl Into<AuthInfo> for User {
    fn into(self) -> AuthInfo {
        AuthInfo {
            id: self.id,
            username: self.username,
            role: self.role,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AuthInfo {
    pub id: i32,
    pub username: String,
    pub role: Role,
}

impl From<AccessToken> for AuthInfo {
    fn from(token: AccessToken) -> Self {
        AuthInfo {
            id: token.sub,
            username: token.username,
            role: token.role,
        }
    }
}
