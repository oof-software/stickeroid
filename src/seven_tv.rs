use std::ffi::OsStr;

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use lazy_static::lazy_static;
use log::{info, warn};
use regex::{Regex, RegexBuilder};

use crate::download;

const USER_AGENT: &str = "\
    Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/110.0.0.0 Safari/537.36";

fn make_url_avif(id: &str) -> String {
    format!("https://cdn.7tv.app/emote/{id}/4x.avif")
}

fn make_url_webp(id: &str) -> String {
    format!("https://cdn.7tv.app/emote/{id}/4x.webp")
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

/// [reqwest::Client] with a user-agent of a normal browser
pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

/// Load a newline-separated list of emote ids from a file
pub async fn ids_from_file<P>(path: P) -> Result<Vec<String>>
where
    P: AsRef<OsStr>,
{
    lazy_static! {
        static ref ID_RE: Regex = RegexBuilder::new(r"^([a-f0-9]{24})\r?$")
            .multi_line(true)
            .build()
            .unwrap();
    }

    let data = tokio::fs::read_to_string(path.as_ref()).await?;

    Ok(ID_RE
        .captures_iter(&data)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect::<Vec<_>>())
}

pub async fn seven_tv_emote(client: &reqwest::Client, id: &str) -> Result<Download> {
    let data = download::download(client, &make_url_avif(id)).await;

    match &data {
        Ok(data) => info!("downloaded 7tv emote `{}` ({})", id, data.len()),
        Err(err) => warn!("couldn't download 7tv emote `{id}`: {err}"),
    }

    Ok(Download::new(id.to_string(), data?))
}

pub async fn seven_tv_emotes<'a, I>(
    client: &reqwest::Client,
    ids: I,
    parllel: usize,
) -> Vec<Result<Download>>
where
    I: IntoIterator,
    I::Item: AsRef<str> + 'a,
{
    futures::stream::iter(ids)
        .map(|id| async move { seven_tv_emote(client, id.as_ref()).await })
        .buffer_unordered(parllel)
        .collect()
        .await
}
