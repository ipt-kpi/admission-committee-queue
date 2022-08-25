use anyhow::Result;
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::fs::OpenOptions;
use std::path::Path;

use crate::queue::Schedule;
use crate::{database, queue};

pub mod date_format;
pub mod time_format;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub token: String,
    pub database_url: String,
    pub max_connections: u32,
    #[serde(with = "date_format")]
    pub schedule: BTreeMap<NaiveDate, Schedule>,
    pub post: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut map = BTreeMap::new();
        let date = NaiveDate::from_ymd(2021, 8, 1);
        let schedule = Schedule {
            start_time: NaiveTime::from_hms(10, 0, 0),
            interval: 30,
            max_enrollee: 50,
        };
        map.insert(date, schedule);
        Config {
            token: "".to_string(),
            database_url: "".to_string(),
            max_connections: 5,
            schedule: map,
            post: "".to_string(),
        }
    }
}

impl Config {
    pub async fn new<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        if let Some(path) = config_path.as_ref().parent() {
            fs::create_dir_all(path)?;
        }

        if !config_path.as_ref().exists() {
            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(config_path)?;

            let config = Self::default();
            serde_json::to_writer_pretty(&file, &config)?;
            return Ok(config);
        }

        let config_file = OpenOptions::new().read(true).open(config_path)?;
        serde_json::from_reader(&config_file).map_err(|error| anyhow::anyhow!(error))
    }

    pub async fn initialize_data(self) -> Result<()> {
        database::initialize(self.max_connections, &self.database_url, self.post.clone()).await?;
        queue::initialize(self.schedule).await?;
        Ok(())
    }
}
