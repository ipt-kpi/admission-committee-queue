use anyhow::{Context, Result};
use captcha::Difficulty;
use std::borrow::Cow;
use teloxide::prelude::*;
use teloxide::types::InputFile;

pub struct Captcha;

impl Captcha {
    pub async fn send(cx: &TransitionIn<AutoSend<Bot>>) -> Result<String> {
        let (answer, file) = captcha::gen(Difficulty::Easy)
            .as_tuple()
            .map(|(answer, data)| (answer, InputFile::memory("captcha", Cow::from(data))))
            .context(anyhow::anyhow!("Failed to generate captcha png"))?;
        cx.answer_photo(file).await?;
        Ok(answer)
    }
}
