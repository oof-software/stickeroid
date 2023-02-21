use std::path::{Path, PathBuf};

use anyhow::Result;
use bytes::Bytes;
use log::{info, warn};

const USER_AGENT: &str = "\
    Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/110.0.0.0 Safari/537.36";

#[derive(Default, Debug)]
pub struct Download {
    pub file_name: PathBuf,
    pub data: Bytes,
}
impl Download {
    pub fn new(file_name: PathBuf, data: Bytes) -> Download {
        Self { file_name, data }
    }
    pub async fn save_to_dir<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref().join(&self.file_name);
        let result = tokio::fs::write(&path, &self.data).await;

        match &result {
            Ok(_) => info!("wrote `{}`", path.display()),
            Err(err) => warn!("couldn't write `{}`: {}", path.display(), err),
        };

        Ok(result?)
    }
}

pub fn client() -> Result<reqwest::Client> {
    Ok(reqwest::ClientBuilder::new()
        .user_agent(USER_AGENT)
        .build()?)
}

pub async fn download(client: &reqwest::Client, url: &str) -> Result<Bytes> {
    Ok(client.get(url).send().await?.bytes().await?)
}
