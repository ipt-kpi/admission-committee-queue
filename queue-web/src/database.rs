use anyhow::Result;
use async_trait::async_trait;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::handlers::user::auth::RegistrationInfo;
use crate::hash;
use crate::model::user::User;

#[derive(Deserialize, Serialize)]
pub enum DatabaseProvider {
    Empty,
    Postgres(PostgresDatabase),
}

#[async_trait]
pub trait Database {
    async fn init(&mut self) -> Result<()>;
    async fn create_user(&self, info: RegistrationInfo) -> Result<()>;
    async fn user_exists(&self, username: &str) -> Result<bool>;
    async fn get_user_by_name(&self, username: &str) -> Result<Option<User>>;
    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>>;

    async fn create_refresh_session(&self, user_id: i32, fingerprint: &str) -> Result<(Uuid, i64)>;
    async fn update_refresh_session(
        &self,
        fingerprint: &str,
        refresh_token: Uuid,
    ) -> Result<(Uuid, i64, i32)>;
    async fn remove_refresh_session(&self, user_id: i32, refresh_token: Uuid) -> Result<()>;
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostgresDatabase {
    database_url: String,
    max_connections: u32,
    #[serde(skip)]
    pub pool: Option<PgPool>,
}

impl Default for PostgresDatabase {
    fn default() -> Self {
        PostgresDatabase {
            database_url: "postgresql://localhost:5432/postgres".to_string(),
            max_connections: 5,
            pool: None,
        }
    }
}

#[async_trait]
impl Database for PostgresDatabase {
    async fn init(&mut self) -> Result<()> {
        self.pool = Some(
            PgPoolOptions::new()
                .max_connections(self.max_connections)
                .connect(&self.database_url)
                .await
                .unwrap(),
        );
        Ok(())
    }

    async fn create_user(&self, info: RegistrationInfo) -> Result<()> {
        let password = hash::hash_password(&info.password)?;
        sqlx::query("INSERT INTO users (username, email, password) VALUES ($1,$2,$3)")
            .bind(info.username)
            .bind(info.email)
            .bind(password)
            .execute(self.pool.as_ref().unwrap())
            .await?;
        Ok(())
    }

    async fn user_exists(&self, username: &str) -> Result<bool> {
        Ok(
            sqlx::query("SELECT exists (SELECT 1 FROM users WHERE username = $1)")
                .bind(username)
                .fetch_one(self.pool.as_ref().unwrap())
                .await?
                .get("exists"),
        )
    }

    async fn get_user_by_name(&self, username: &str) -> Result<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(self.pool.as_ref().unwrap())
                .await?,
        )
    }

    async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(self.pool.as_ref().unwrap())
                .await?,
        )
    }

    async fn create_refresh_session(&self, user_id: i32, fingerprint: &str) -> Result<(Uuid, i64)> {
        let sessions_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM refresh_sessions WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(self.pool.as_ref().unwrap())
                .await?
                .get("count");

        if sessions_count >= 5 {
            sqlx::query("DELETE FROM refresh_sessions WHERE ctid IN(SELECT ctid FROM refresh_sessions WHERE user_id = $1 AND fingerprint = $2 ORDER BY created_at DESC LIMIT 1)")
                .bind(user_id)
                .bind(fingerprint)
                .execute(self.pool.as_ref().unwrap())
                .await?;
        }

        let timestamp = (Utc::now() + Duration::weeks(2)).timestamp();
        let refresh_token = sqlx::query(
            "INSERT INTO refresh_sessions (user_id, fingerprint, expires_in) VALUES ($1,$2,$3) RETURNING refresh_token",
        )
            .bind(user_id)
            .bind(fingerprint)
            .bind(timestamp)
            .fetch_one(self.pool.as_ref().unwrap())
            .await?
            .get("refresh_token");
        Ok((refresh_token, timestamp))
    }

    async fn update_refresh_session(
        &self,
        fingerprint: &str,
        refresh_token: Uuid,
    ) -> Result<(Uuid, i64, i32)> {
        let row: PgRow = sqlx::query(
            "DELETE FROM refresh_sessions WHERE refresh_token = $1 RETURNING expires_in, fingerprint, user_id",
        )
        .bind(refresh_token)
        .fetch_one(self.pool.as_ref().unwrap())
        .await?;
        if fingerprint != &row.get::<String, _>("fingerprint") {
            return Err(anyhow::anyhow!("Failed to verify fingerprint"));
        }

        if Utc::now().timestamp() >= row.get("expires_in") {
            return Err(anyhow::anyhow!("This session has been expired"));
        }
        let user_id = row.get("user_id");
        self.create_refresh_session(user_id, fingerprint)
            .await
            .map(|session| (session.0, session.1, user_id))
    }

    async fn remove_refresh_session(&self, user_id: i32, refresh_token: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM refresh_sessions WHERE user_id = $1 and refresh_token = $2")
            .bind(user_id)
            .bind(refresh_token)
            .execute(self.pool.as_ref().unwrap())
            .await?;
        Ok(())
    }
}

impl DatabaseProvider {
    pub async fn init(&mut self) -> Result<()> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database.init().await,
        }
    }
    pub async fn create_user(&self, info: RegistrationInfo) -> Result<()> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database.create_user(info).await,
        }
    }
    pub async fn user_exists(&self, username: &str) -> Result<bool> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database.user_exists(username).await,
        }
    }
    pub async fn get_user_by_name(&self, username: &str) -> Result<Option<User>> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database.get_user_by_name(username).await,
        }
    }
    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database.get_user_by_id(user_id).await,
        }
    }
    pub async fn create_refresh_session(
        &self,
        user_id: i32,
        fingerprint: &str,
    ) -> Result<(Uuid, i64)> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => {
                database.create_refresh_session(user_id, fingerprint).await
            }
        }
    }
    pub async fn update_refresh_session(
        &self,
        fingerprint: &str,
        refresh_token: Uuid,
    ) -> Result<(Uuid, i64, i32)> {
        Ok(match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => database
                .update_refresh_session(fingerprint, refresh_token)
                .await
                .unwrap(),
        })
    }
    pub async fn remove_refresh_session(&self, user_id: i32, refresh_token: Uuid) -> Result<()> {
        match self {
            DatabaseProvider::Empty => unimplemented!(),
            DatabaseProvider::Postgres(database) => {
                database
                    .remove_refresh_session(user_id, refresh_token)
                    .await
            }
        }
    }
}
