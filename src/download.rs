use std::ffi::OsStr;

use anyhow::Result;
use bytes::Bytes;

const USER_AGENT: &str = "\
    Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) \
    Chrome/110.0.0.0 Safari/537.36";

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

pub async fn download(client: &reqwest::Client, url: &str) -> Result<Bytes> {
    Ok(client.get(url).send().await?.bytes().await?)
}

pub async fn save_to_file<P>(bytes: &Bytes, dst: P) -> Result<()>
where
    P: AsRef<OsStr>,
{
    Ok(tokio::fs::write(dst.as_ref(), bytes).await?)
}
