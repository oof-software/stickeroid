use anyhow::Result;
use futures::StreamExt;
use log::info;
use structopt::StructOpt;
use walkdir::WalkDir;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::binaries::Binaries;
use crate::download::Client;
use crate::emote_ext::{BttvId, EmoteId, SevenTvId};
use crate::opt::Opt;
use crate::webp;

#[derive(Debug, Clone)]
pub struct Context {
    pub opt: Arc<Opt>,
    pub client: Client,
    pub bin: Arc<Binaries>,
}

impl Context {
    pub fn new() -> Result<Context> {
        Ok(Context {
            opt: Arc::new(Opt::from_args()),
            client: Client::new(),
            bin: Arc::new(Binaries::from_env()?),
        })
    }

    pub fn as_seven_tv_ids(&self) -> &[SevenTvId] {
        &self.opt.seven_tv_ids
    }
    pub fn as_bttv_ids(&self) -> &[BttvId] {
        &self.opt.bttv_ids
    }
    pub fn to_emote_ids(&self) -> Vec<EmoteId> {
        let bttv_ids = self.as_bttv_ids();
        let seven_tv_ids = self.as_seven_tv_ids();
        let mut ids = Vec::with_capacity(bttv_ids.len() + seven_tv_ids.len());
        ids.extend(bttv_ids.iter().map(EmoteId::from));
        ids.extend(seven_tv_ids.iter().map(EmoteId::from));
        ids
    }

    async fn download_emote(&self, id: EmoteId) -> Result<()> {
        let dl = self.client.get_emote(id).await?;
        dl.write_to(&self.opt.download_dir).await?;
        info!("downloaded emote `{id:?}`");
        Ok(())
    }
    pub async fn download_emotes(&self, ids: &[EmoteId]) -> Result<()> {
        let mut iter = futures::stream::iter(ids)
            .map(|id| self.download_emote(*id))
            .buffer_unordered(5);
        while let Some(r) = iter.next().await {
            if let Err(e) = r {
                return Err(e);
            }
        }
        Ok(())
    }

    async fn webp_info(&self, path: impl AsRef<Path>) -> Result<webp::WebpInfo> {
        let info = self.bin.webp_info.info(path.as_ref()).await?;
        info!("got webp_info for `{}`", path.as_ref().display());
        Ok(info)
    }
    pub async fn webp_infos(&self, paths: &[PathBuf]) -> Result<Vec<webp::WebpInfo>> {
        let mut infos = Vec::with_capacity(paths.len());
        let mut iter = futures::stream::iter(paths)
            .map(|path| self.webp_info(path))
            .buffer_unordered(5);
        while let Some(info) = iter.next().await {
            infos.push(info?);
        }
        Ok(infos)
    }

    pub async fn downloaded_emotes(&self) -> Vec<PathBuf> {
        let dl_dir = self.opt.download_dir.clone();
        tokio::task::spawn_blocking(move || -> Vec<PathBuf> {
            WalkDir::new(dl_dir)
                .max_depth(1)
                .min_depth(1)
                .into_iter()
                .filter_map(|entry| Some(entry.ok()?.into_path()))
                .collect()
        })
        .await
        .unwrap()
    }
}
