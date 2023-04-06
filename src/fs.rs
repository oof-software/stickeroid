use std::path::Path;

use anyhow::Result;
use simple_error::simple_error;

pub async fn assert_dir(path: impl AsRef<Path>) -> Result<()> {
    match tokio::fs::metadata(path.as_ref()).await {
        Ok(meta) => {
            if meta.is_dir() {
                Ok(())
            } else {
                Err(simple_error!("path exists but is not a directory").into())
            }
        }
        Err(_) => Ok(tokio::fs::create_dir(path.as_ref()).await?),
    }
}

pub async fn is_dir_empty(path: impl AsRef<Path>) -> Result<bool> {
    let mut dir = tokio::fs::read_dir(path.as_ref()).await?;
    Ok(!dir.next_entry().await?.is_some())
}
