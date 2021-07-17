use anyhow::Result;
use chrono::{Duration, NaiveDate, Utc};
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::handlers::user::auth::RegistrationInfo;
use crate::hash;
use crate::model::enrollee::{Enrollee, Status};
use crate::model::user::User;

pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(max_connections: u32, database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(database_url)
            .await?;
        Ok(Database { pool })
    }

    pub async fn create_user(&self, info: RegistrationInfo) -> Result<()> {
        let password = hash::hash_password(&info.password)?;
        sqlx::query("INSERT INTO users (username, email, password) VALUES ($1,$2,$3)")
            .bind(info.username)
            .bind(info.email)
            .bind(password)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn user_exists(&self, username: &str) -> Result<bool> {
        Ok(
            sqlx::query("SELECT exists (SELECT 1 FROM users WHERE username = $1)")
                .bind(username)
                .fetch_one(&self.pool)
                .await?
                .get("exists"),
        )
    }

    pub async fn get_user_by_name(&self, username: &str) -> Result<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
                .bind(username)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> Result<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?,
        )
    }

    pub async fn create_refresh_session(
        &self,
        user_id: i32,
        fingerprint: &str,
    ) -> Result<(Uuid, i64)> {
        let sessions_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM refresh_sessions WHERE user_id = $1")
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?
                .get("count");

        if sessions_count >= 5 {
            sqlx::query("DELETE FROM refresh_sessions WHERE ctid IN(SELECT ctid FROM refresh_sessions WHERE user_id = $1 AND fingerprint = $2 ORDER BY created_at DESC LIMIT 1)")
                .bind(user_id)
                .bind(fingerprint)
                .execute(&self.pool)
                .await?;
        }

        let timestamp = (Utc::now() + Duration::weeks(2)).timestamp();
        let refresh_token = sqlx::query(
            "INSERT INTO refresh_sessions (user_id, fingerprint, expires_in) VALUES ($1,$2,$3) RETURNING refresh_token",
        )
            .bind(user_id)
            .bind(fingerprint)
            .bind(timestamp)
            .fetch_one(&self.pool)
            .await?
            .get("refresh_token");
        Ok((refresh_token, timestamp))
    }

    pub async fn update_refresh_session(
        &self,
        fingerprint: &str,
        refresh_token: Uuid,
    ) -> Result<(Uuid, i64, i32)> {
        let row: PgRow = sqlx::query(
            "DELETE FROM refresh_sessions WHERE refresh_token = $1 RETURNING expires_in, fingerprint, user_id",
        )
            .bind(refresh_token)
            .fetch_one(&self.pool)
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

    pub async fn remove_refresh_session(&self, user_id: i32, refresh_token: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM refresh_sessions WHERE user_id = $1 and refresh_token = $2")
            .bind(user_id)
            .bind(refresh_token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_dates(&self) -> Result<Vec<NaiveDate>> {
        sqlx::query("SELECT DISTINCT date FROM queue ORDER BY date")
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|dates| dates.iter().map(|row| row.get(0)).collect())
    }

    pub async fn get_enrollees(&self, dates: Vec<NaiveDate>) -> Result<Vec<Enrollee>> {
        sqlx::query_as(
            "SELECT id, last_name, name, patronymic, date, time, status, username, phone_number 
                FROM enrollee INNER JOIN queue ON enrollee.id = queue.enrollee
                WHERE (SELECT date = ANY ($1))
                ORDER BY date, time",
        )
        .bind(dates)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| anyhow::anyhow!(error))
    }

    pub async fn change_status(&self, id: i64, status: Status) -> Result<()> {
        sqlx::query("UPDATE queue SET status = $1 WHERE enrollee = $2")
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_enrollee(&self, enrollee: Enrollee) -> Result<()> {
        sqlx::query(
            "UPDATE enrollee SET last_name = $1, name = $2, patronymic = $3, username = $4, phone_number = $5
                WHERE id = $6"
        )
            .bind(enrollee.last_name)
            .bind(enrollee.name)
            .bind(enrollee.patronymic)
            .bind(enrollee.username)
            .bind(enrollee.phone_number)
            .bind(enrollee.id)
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE queue SET date = $1, time = $2, status = $3 WHERE enrollee = $4")
            .bind(enrollee.date)
            .bind(enrollee.time)
            .bind(enrollee.status)
            .bind(enrollee.id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
