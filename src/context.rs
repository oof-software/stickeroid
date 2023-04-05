use anyhow::Result;
use futures::StreamExt;
use log::info;
use structopt::StructOpt;
use walkdir::WalkDir;

use std::path::PathBuf;
use std::sync::Arc;

use crate::binaries::Binaries;
use crate::download::Client;
use crate::emote_ext::{BttvId, EmoteId, EmoteIdExt, SevenTvId};
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

    pub async fn download_emote(&self, id: EmoteId) -> Result<()> {
        let dl = self.client.get_emote(id).await?;
        dl.write_to(&self.opt.download_dir).await?;
        info!("downloaded emote `{id:?}`");
        Ok(())
    }
    pub async fn download_emotes(&self, ids: &[EmoteId]) -> Result<()> {
        let mut iter = futures::stream::iter(ids)
            .map(|id| self.download_emote(*id))
            .buffer_unordered(5);
        while let Some(result) = iter.next().await {
            if let err @ Err(_) = result {
                return err;
            }
        }
        Ok(())
    }

    pub async fn webp_info(&self, id: EmoteId) -> Result<webp::WebpInfo> {
        let path = self.opt.download_dir.join(id.to_file_name());
        let info = self.bin.webp_info.info(&path).await?;
        info!("got webp_info for emote `{id:?}`");
        Ok(info)
    }
    pub async fn webp_infos(&self, ids: &[EmoteId]) -> Result<Vec<webp::WebpInfo>> {
        let mut infos = Vec::with_capacity(ids.len());
        let mut iter = futures::stream::iter(ids)
            .map(|id| self.webp_info(*id))
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

    pub async fn extract_frames_single(&self, id: EmoteId) -> Result<()> {
        let src = self.opt.download_dir.join(id.to_file_name());
        let dst = self.opt.frames_dir.join(id.to_string());
        crate::fs::assert_dir(&dst).await?;
        self.bin.anim_dump.dump_frames(src, dst).await?;
        info!("extracted frames for emote `{id:?}`");
        Ok(())
    }

    pub async fn extract_frames(&self, ids: &[EmoteId]) -> Result<()> {
        let mut iter = futures::stream::iter(ids)
            .map(|id| self.extract_frames_single(*id))
            .buffer_unordered(5);
        while let Some(result) = iter.next().await {
            if let err @ Err(_) = result {
                return err;
            }
        }
        Ok(())
    }

    // https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
    const STATIC_SIZE_LIMIT: usize = 100 * 1024;
    const ANIMATED_SIZE_LIMIT: usize = 500 * 1024;

    pub fn partition_infos_valid(
        infos: &mut [webp::WebpInfo],
    ) -> (&[webp::WebpInfo], &[webp::WebpInfo]) {
        let partition_point = infos
            .iter_mut()
            .partition_in_place(webp::WebpInfo::is_valid);
        infos.split_at(partition_point)
    }
}
