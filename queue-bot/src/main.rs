use anyhow::Result;
use std::sync::Arc;
use teloxide::prelude::*;

use crate::config::Config;
use crate::database::notifier::Notifier;
use crate::database::Database;
use crate::dialogue::Dialogue;

mod captcha;
mod config;
mod database;
mod dialogue;
mod queue;
mod user;

#[tokio::main]
async fn main() {
    let config = Config::get_config("config.json.bak-bot")
        .await
        .expect("Failed to initialize config");
    let bot = Bot::from_env().auto_send();
    let notifier = Notifier::new(&config.database_url, bot.clone())
        .await
        .expect("Failed to initialize notifier");
    config
        .initialize_data()
        .await
        .expect("Failed to initialize all global data");
    tokio::try_join!(run(bot), notifier.run()).expect("Something get wrong with main tasks");
}
type In = DialogueWithCx<AutoSend<Bot>, Message, Dialogue, anyhow::Error>;

async fn run(bot: AutoSend<Bot>) -> Result<()> {
    teloxide::enable_logging!();
    log::info!("Starting queue_bot...");

    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                let dialogue = dialogue.expect("std::convert::Infallible");
                handle_message(cx, dialogue)
                    .await
                    .expect("Something wrong with the bot!")
            },
            Arc::new(Database::global()),
        ))
        .dispatch()
        .await;
    Ok(())
}

async fn handle_message(
    cx: UpdateWithCx<AutoSend<Bot>, Message>,
    dialogue: Dialogue,
) -> TransitionOut<Dialogue> {
    match cx.update.text().map(ToOwned::to_owned) {
        None => {
            cx.answer("Send me a text message.").await?;
            next(dialogue)
        }
        Some(ans) => dialogue.react(cx, ans).await,
    }
}
