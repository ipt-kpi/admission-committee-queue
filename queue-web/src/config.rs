use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;

use crate::database::{DatabaseProvider, PostgresDatabase};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub database: DatabaseProvider,
    pub address: String,
    pub recaptcha_token: String,
}

impl Config {
    pub fn new(config_path: &str) -> Result<Self> {
        let config_file = OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .open(config_path)?;
        match serde_json::from_reader(&config_file) {
            Ok(config) => Ok(config),
            Err(_) => {
                let config = Self::default();
                serde_json::to_writer_pretty(&config_file, &config)?;
                Ok(config)
            }
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            address: "127.0.0.1:3030".to_string(),
            database: DatabaseProvider::Postgres(PostgresDatabase::default()),
            recaptcha_token: "".to_string(),
        }
    }
}
