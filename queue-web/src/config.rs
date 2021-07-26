use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub address: String,
    pub recaptcha_token: String,
    pub public_key: String,
    pub private_key: String,
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
            database_url: "postgresql://localhost:5432/postgres".to_string(),
            max_connections: 5,
            address: "127.0.0.1:3030".to_string(),
            recaptcha_token: "".to_string(),
            public_key: "".to_string(),
            private_key: "".to_string(),
        }
    }
}
