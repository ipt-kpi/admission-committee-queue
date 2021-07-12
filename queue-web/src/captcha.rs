use anyhow::{Context, Result};
use hyper::body;
use hyper::{Body, Client, Uri};
use hyper_tls::HttpsConnector;
use serde_json::Value;
use warp::Buf;

pub struct ReCaptcha {
    token: String,
}

impl ReCaptcha {
    pub fn new(token: String) -> ReCaptcha {
        ReCaptcha { token }
    }
    pub async fn check(&self, client_token: &str) -> Result<bool> {
        let uri: Uri = format!(
            "https://www.google.com/recaptcha/api/siteverify?secret={}&response={}",
            self.token, client_token
        )
        .parse()?;
        let response = Client::builder()
            .build::<_, Body>(HttpsConnector::new())
            .get(uri)
            .await?;
        Ok(
            serde_json::from_reader::<_, Value>(body::aggregate(response).await?.reader())?
                .get("success")
                .context("Failed to get success field from recaptcha response")?
                .as_bool()
                .context("Failed to convert success field to bool")?,
        )
    }
}
