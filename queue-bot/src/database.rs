use anyhow::Result;
use chrono::{Duration, NaiveDate, NaiveTime};
use futures::future::BoxFuture;
use once_cell::sync::OnceCell;
use serde::de::DeserializeOwned;
use serde::Serialize;
use sqlx::postgres::types::PgInterval;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Row};
use std::convert::TryFrom;
use std::fmt::{Debug, Display};
use std::sync::Arc;
use teloxide::dispatching::dialogue::serializer::Json;
use teloxide::dispatching::dialogue::{Serializer, Storage};

use crate::user::Enrollee;

static INSTANCE: OnceCell<Database<Json>> = OnceCell::new();

pub struct Database<S> {
    pool: PgPool,
    serializer: S,
}

pub async fn initialize(max_connections: u32, url: &str) -> Result<()> {
    INSTANCE
        .set(Database {
            pool: PgPoolOptions::new()
                .max_connections(max_connections)
                .connect(url)
                .await?,
            serializer: Json,
        })
        .map_err(|_| anyhow::anyhow!("Failed to initialize database!"))
}

impl Database<Json> {
    pub fn global() -> &'static Database<Json> {
        INSTANCE.get().expect("Pool isn't initialized")
    }

    pub async fn register(&self, enrollee: Enrollee) -> Result<()> {
        println!("{}", enrollee.id);
        sqlx::query("INSERT INTO enrollee (id, username, name, patronymic, last_name, phone_number) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(enrollee.id)
            .bind(enrollee.username)
            .bind(enrollee.name)
            .bind(enrollee.patronymic)
            .bind(enrollee.last_name)
            .bind(enrollee.phone_number)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_occupied_time(
        &self,
        _interval: (NaiveTime, NaiveTime),
    ) -> Result<Vec<NaiveTime>> {
        unimplemented!()
    }

    pub async fn is_banned(&self, id: i64) -> Result<bool> {
        sqlx::query("SELECT banned FROM enrollee WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|optional_row| optional_row.map(|row| row.get(0)).unwrap_or(false))
    }

    pub async fn is_enrollee_valid(
        &self,
        last_name: &str,
        name: &str,
        patronymic: &str,
    ) -> Result<bool> {
        sqlx::query("SELECT * FROM is_enrollee_valid($1, $2, $3)")
            .bind(last_name)
            .bind(name)
            .bind(patronymic)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|row| row.get(0))
    }

    pub async fn get_intervals(
        &self,
        date: NaiveDate,
        time: NaiveTime,
        max_enrollee: u16,
        interval: u16,
    ) -> Result<Vec<String>> {
        let interval = PgInterval::try_from(Duration::minutes(interval as i64))
            .map_err(|error| anyhow::anyhow!(error))?;
        sqlx::query("SELECT * FROM get_intervals($1,$2,$3,$4)")
            .bind(date)
            .bind(time)
            .bind(max_enrollee as i32)
            .bind(interval)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|intervals| intervals.iter().map(|row| row.get(0)).collect())
    }

    pub async fn get_intervals_between(
        &self,
        date: NaiveDate,
        time: NaiveTime,
        max_enrollee: u16,
        interval: u16,
        first_time: NaiveTime,
        second_time: NaiveTime,
    ) -> Result<Vec<String>> {
        let interval = PgInterval::try_from(Duration::minutes(interval as i64))
            .map_err(|error| anyhow::anyhow!(error))?;
        sqlx::query(
            "SELECT to_char(time, 'HH24:MI') FROM get_relevant_time($1,$2,$3,$4) as time WHERE time BETWEEN $5 AND $6",
        )
        .bind(date)
        .bind(time)
        .bind(max_enrollee as i32)
        .bind(interval)
        .bind(first_time)
        .bind(second_time)
        .fetch_all(&self.pool)
        .await
        .map_err(|error| anyhow::anyhow!(error))
        .map(|intervals| intervals.iter().map(|row| row.get(0)).collect())
    }

    pub async fn check_time(&self, date: NaiveDate, time: NaiveTime) -> Result<bool> {
        sqlx::query("SELECT * FROM exists(SELECT 1 FROM queue WHERE date = $1 AND time = $2)")
            .bind(date)
            .bind(time)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|row| row.get(0))
    }

    pub async fn register_in_queue(
        &self,
        id: i64,
        date: NaiveDate,
        time: NaiveTime,
    ) -> Result<bool> {
        sqlx::query("SELECT * FROM register_in_queue($1,$2,$3)")
            .bind(id)
            .bind(date)
            .bind(time)
            .fetch_one(&self.pool)
            .await
            .map_err(|error| anyhow::anyhow!(error))
            .map(|row| row.get(0))
    }
}

impl<S, D> Storage<D> for &'static Database<S>
where
    S: Send + Sync + Serializer<D> + 'static,
    D: Send + Serialize + DeserializeOwned + 'static,
    <S as Serializer<D>>::Error: Debug + Display,
{
    type Error = anyhow::Error;

    fn remove_dialogue(
        self: Arc<Self>,
        chat_id: i64,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            Ok(match get_dialogue(&self.pool, chat_id).await? {
                Some(d) => {
                    let prev_dialogue = self.serializer.deserialize(&d).map_err(|error| {
                        anyhow::anyhow!("dialogue serialization error: {}", error)
                    })?;
                    sqlx::query("DELETE FROM teloxide_dialogues WHERE chat_id = $1")
                        .bind(chat_id)
                        .execute(&self.pool)
                        .await?;
                    Some(prev_dialogue)
                }
                _ => None,
            })
        })
    }

    fn update_dialogue(
        self: Arc<Self>,
        chat_id: i64,
        dialogue: D,
    ) -> BoxFuture<'static, Result<Option<D>, Self::Error>> {
        Box::pin(async move {
            let prev_dialogue = get_dialogue(&self.pool, chat_id)
                .await?
                .map(|d| {
                    self.serializer
                        .deserialize(&d)
                        .map_err(|error| anyhow::anyhow!("Database deserialize error: {}", error))
                })
                .transpose()?;
            let upd_dialogue = self
                .serializer
                .serialize(&dialogue)
                .map_err(|error| anyhow::anyhow!("Database serialize error: {}", error))?;
            self.pool
                .acquire()
                .await?
                .execute(
                    sqlx::query(
                        r#"
            INSERT INTO teloxide_dialogues VALUES ($1, $2)
            ON CONFLICT(chat_id) DO UPDATE SET dialogue=excluded.dialogue
                                "#,
                    )
                    .bind(chat_id)
                    .bind(upd_dialogue),
                )
                .await
                .unwrap();
            Ok(prev_dialogue)
        })
    }
}

async fn get_dialogue(pool: &PgPool, chat_id: i64) -> Result<Option<Box<Vec<u8>>>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct DialogueDbRow {
        dialogue: Vec<u8>,
    }

    Ok(sqlx::query_as::<_, DialogueDbRow>(
        "SELECT dialogue FROM teloxide_dialogues WHERE chat_id = $1",
    )
    .bind(chat_id)
    .fetch_optional(pool)
    .await?
    .map(|r| Box::new(r.dialogue)))
}
