use anyhow::{Context, Error, Result};
use futures::{stream, StreamExt, TryStreamExt};
use scraper::{Html, Selector};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub struct Parser {
    url: String,
}

impl Parser {
    pub fn new(year: u8) -> Self {
        let url = format!("https://abit-poisk.org.ua/rate20{}/direction/", year);
        Parser { url }
    }
    pub async fn get_info(&self, code: &str, file: &mut File) -> Result<()> {
        let url = &format!("{}{}", &self.url, code);
        let document = Self::get_html(url).await?;
        let pages = Self::get_pages(&document).await?;

        if pages == 1 {
            let names = Self::get_names(document).await;
            file.write_all(names.join("\n").as_bytes()).await?;
            return Ok(());
        }

        let stream = stream::iter(2..=pages)
            .map(|page| async move { Self::get_html(&format!("{}/?page={}", url, page)).await })
            .buffer_unordered(3)
            .chain(stream::once(async { Ok::<Html, Error>(document) }))
            .map(|document| async {
                match document {
                    Ok(document) => Ok(Self::get_names(document).await),
                    Err(error) => Err(error),
                }
            })
            .buffer_unordered(3);

        futures::pin_mut!(stream);
        while let Some(names) = stream.try_next().await? {
            file.write_all((names.join("\n") + "\n").as_bytes()).await?;
        }
        Ok(())
    }

    async fn get_pages(document: &Html) -> Result<u8> {
        let div_selector =
            Selector::parse(r#"div[class=""]"#).expect("Failed to parse div selector");
        let div = document
            .select(&div_selector)
            .last()
            .expect("Failed to parse page number");
        let selector =
            Selector::parse(r#"a[data-scroll-on-load=""].btn.btn-default.ajax.secondary-text"#)
                .expect("Failed to parse page selector");
        let mut elements = div.select(&selector);
        Ok(match elements.by_ref().count() {
            0 => 1,
            x if x < 5 => (x - 1) as u8,
            x => elements
                .skip(x - 2)
                .next()
                .context("Failed to get pages number")?
                .inner_html()
                .parse()?,
        })
    }

    async fn get_html(url: &str) -> Result<Html> {
        let response = reqwest::get(url).await?.text().await?;
        Ok(Html::parse_document(&response))
    }

    async fn get_names(document: Html) -> Vec<String> {
        let selector =
            Selector::parse(r#"a[href^="/#search-"]"#).expect("Failed to parse name selector");
        document
            .select(&selector)
            .map(|element| element.inner_html().trim().to_string())
            .collect::<Vec<_>>()
    }
}
