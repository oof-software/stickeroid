use std::path::{Path, PathBuf};

use anyhow::Result;
use futures::StreamExt;
use lazy_regex::lazy_regex;
use log::{info, warn};

use crate::download::{download, Download};

fn make_url_webp(id: &str) -> String {
    format!("https://cdn.betterttv.net/emote/{id}/3x.webp")
}
fn make_file_name_webp(id: &str) -> PathBuf {
    Path::new(id).with_extension(".webp")
}

/// Load a newline-separated list of emote ids from a file
pub async fn ids_from_file<P>(path: P) -> Result<Vec<String>>
where
    P: AsRef<Path>,
{
    let data = tokio::fs::read_to_string(path).await?;

    Ok(lazy_regex!(r"^([a-f0-9]{24})\r?$"m)
        .captures_iter(&data)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect::<Vec<_>>())
}

pub async fn bttv_emote(client: &reqwest::Client, id: &str) -> Result<Download> {
    let data = download(client, &make_url_webp(id)).await;

    match &data {
        Ok(data) => info!("downloaded bttv emote `{}` ({})", id, data.len()),
        Err(err) => warn!("couldn't download bttv emote `{id}`: {err}"),
    }

    Ok(Download::new(make_file_name_webp(id), data?))
}

pub async fn bttv_emotes<I>(
    client: &reqwest::Client,
    ids: I,
    parllel: usize,
) -> Vec<Result<Download>>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    futures::stream::iter(ids)
        .map(|id| async move { bttv_emote(client, id.as_ref()).await })
        .buffer_unordered(parllel)
        .collect()
        .await
}
