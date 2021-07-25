use anyhow::Result;
use futures::TryStreamExt;
use serde_json::Value;
use sqlx::postgres::PgListener;
use teloxide::prelude::{AutoSend, Requester};
use teloxide::Bot;

pub struct Notifier {
    listener: PgListener,
    bot: AutoSend<Bot>,
}

impl Notifier {
    pub async fn new(url: &str, bot: AutoSend<Bot>) -> Result<Self> {
        Ok(Notifier {
            listener: PgListener::connect(url).await?,
            bot,
        })
    }

    pub async fn run(mut self) -> Result<()> {
        self.listener.listen_all(vec!["queue_status"]).await?;
        let mut stream = self.listener.into_stream();
        while let Some(notification) = stream.try_next().await? {
            let payload: Value = serde_json::from_str(notification.payload())?;
            if let Some(id) = payload.get("f1").map(|id| id.as_i64()).flatten() {
                if let Some(count) = payload.get("f2").map(|count| count.as_i64()).flatten() {
                    let message = match count {
                        0 => {
                            format!("Підійшла ваша черга!")
                        }
                        count => {
                            format!("Перед вами в черзі перебуває {} людина(-и, -ей)", count)
                        }
                    };
                    if let Err(error) = self.bot.send_message(id, message).await {
                        log::error!("Failed to send notification message: {}", error);
                    }
                }
            }
        }
        Ok(())
    }
}
