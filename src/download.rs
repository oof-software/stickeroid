use std::path::{Path, PathBuf};

use anyhow::Result;
use bytes::Bytes;

use crate::emote_ext::{EmoteId, EmoteIdExt};

const USER_AGENT: &str = "\
    Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/110.0.0.0 Safari/537.36";

#[derive(Debug, Clone)]
pub struct Client(reqwest::Client);

impl Client {
    pub fn new() -> Client {
        Client(
            reqwest::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        )
    }

    pub async fn get(&self, url: &str, file_name: impl AsRef<Path>) -> Result<Download> {
        let resp = self.0.get(url).send().await?;
        let bytes = resp.bytes().await?;
        Ok(Download::new(file_name.as_ref().to_owned(), bytes))
    }
    pub async fn get_emote(&self, emote: EmoteId) -> Result<Download> {
        let resp = self.0.get(emote.to_url()).send().await?;
        let bytes = resp.bytes().await?;
        Ok(Download::new(emote.to_file_name(), bytes))
    }
}

#[derive(Default, Debug)]
pub struct Download {
    pub file_name: PathBuf,
    pub data: Bytes,
}
impl Download {
    pub fn new(file_name: PathBuf, data: Bytes) -> Download {
        Self { file_name, data }
    }
    pub async fn write_to(&self, dir: impl AsRef<Path>) -> Result<()> {
        let path = dir.as_ref().join(&self.file_name);
        Ok(tokio::fs::write(&path, &self.data).await?)
    }
}
