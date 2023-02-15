use std::ffi::OsStr;

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use log::{trace, warn};

const USER_AGENT: &str = "\
    Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/110.0.0.0 Safari/537.36";

fn make_url(id: &str) -> String {
    format!("https://cdn.7tv.app/emote/{id}/4x.avif")
}

#[derive(Default, Debug)]
pub struct Download {
    pub name: String,
    pub data: Bytes,
}
impl Download {
    pub fn new(name: String, data: Bytes) -> Download {
        Self { name, data }
    }
}

pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

pub async fn seven_tv_emote(client: &reqwest::Client, id: &str) -> Result<Download> {
    let data = client.get(make_url(id)).send().await?.bytes().await;

    match &data {
        Ok(data) => trace!("downloaded 7tv emote `{}` ({})", id, data.len()),
        Err(err) => warn!("couldn't download 7tv emote `{id}`: {err}"),
    }

    Ok(Download::new(id.to_string(), data?))
}

pub async fn seven_tv_emotes<'a, I>(
    client: &reqwest::Client,
    ids: I,
    n: usize,
) -> Vec<Result<Download>>
where
    I: IntoIterator<Item = &'a str>,
{
    futures::stream::iter(ids)
        .map(|id| seven_tv_emote(client, id))
        .buffer_unordered(n)
        .collect()
        .await
}
