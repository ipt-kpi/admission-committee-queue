use anyhow::Result;
use log::info;
use std::net::SocketAddrV4;
use warp::Filter;

use crate::captcha::ReCaptcha;
use crate::config::Config;
use crate::database::Database;
use crate::jwt::Jwt;

mod captcha;
mod config;
mod database;
mod filter;
mod handlers;
mod hash;
mod jwt;
mod mail;
mod model;
mod reject;

pub struct Application {
    database: Database,
    jwt: Jwt,
    recaptcha: ReCaptcha,
}
impl Application {
    async fn new(config: Config) -> Result<Self> {
        Ok(Application {
            database: Database::new(config.max_connections, &config.database_url).await?,
            jwt: Jwt::new(config.public_key, config.private_key)?,
            recaptcha: ReCaptcha::new(config.recaptcha_token),
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();
    info!("Read configuration...");
    let config = Config::new("config.json")?;
    let address: SocketAddrV4 = config.address.parse()?;

    let app: &'static Application = Box::leak(Box::new(Application::new(config).await?));
    let prefix = warp::path!("api" / ..);
    let cors = warp::cors()
        .allow_any_origin()
        .allow_credentials(true)
        .allow_header("content-type")
        .allow_header("authorization")
        .allow_methods(vec!["GET", "POST", "DELETE", "OPTIONS"]);
    info!("IPT-Queue starting...");
    Ok(
        warp::serve(prefix.and(filter::routes(app).recover(reject::recover).with(cors)))
            .run(address)
            .await,
    )
}
