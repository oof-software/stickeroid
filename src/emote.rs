use anyhow::Result;
use futures::StreamExt;
use log::{info, warn};
use simple_error::simple_error;

use crate::binaries::Img2WebpFrame;
use crate::context::Context;
use crate::convert::ConversionOptions;
use crate::emote_ext::EmoteId;
use crate::file_sequence::FileSequence;
use crate::webp::WebpInfo;

#[derive(Debug)]
pub struct Emote {
    pub id: EmoteId,
    pub info: WebpInfo,
    pub raw_frames: FileSequence,
    pub resized_frames: FileSequence,
}

pub struct BatchElement {
    pub id: EmoteId,
    pub result: Result<Emote>,
}

impl Emote {
    pub async fn download(ctx: &Context, id: EmoteId) -> Result<()> {
        let dl_path = ctx.download_path(id);

        if tokio::fs::metadata(&dl_path).await.is_ok() {
            warn!("emote `{id:?}` was already downloaded");
            return Ok(());
        }

        let dl = ctx.client.get_emote(id).await?;
        dl.write_to(dl_path).await?;

        info!("downloaded emote `{id:?}`");
        Ok(())
    }

    pub async fn webp_info(ctx: &Context, id: EmoteId) -> Result<WebpInfo> {
        let info = ctx.bin.webp_info.info(ctx.download_path(id)).await?;
        info!("got webp_info for emote `{id:?}`");
        Ok(info)
    }

    pub async fn extract_frames(ctx: &Context, id: EmoteId) -> Result<FileSequence> {
        let dst = ctx.raw_frames_path(id);
        crate::fs::assert_dir(&dst).await?;

        if !crate::fs::is_dir_empty(&dst).await? {
            warn!("frames for emote `{id:?}` are already extracted");
        } else {
            let src = ctx.download_path(id);
            ctx.bin.anim_dump.dump_frames(&src, &dst).await?;
            info!("extracted frames for emote `{id:?}`");
        }

        crate::file_sequence::file_sequence(&dst).await
    }

    pub async fn resize_frames(ctx: &Context, id: EmoteId) -> Result<FileSequence> {
        let dst = ctx.resized_frames_path(id);
        crate::fs::assert_dir(&dst).await?;

        if !crate::fs::is_dir_empty(&dst).await? {
            warn!("frames for emote `{id:?}` are already resized");
        } else {
            let src = ctx.raw_frames_path(id).join("%04d.png");
            let dst = dst.join("%04d.png");
            ctx.bin.ffmpeg.resize_images(src, dst).await?;
            info!("resized frames for emote `{id:?}`");
        }

        crate::file_sequence::file_sequence(&dst).await
    }

    async fn to_sticker_static(&self, ctx: &Context) -> Result<()> {
        let src_file_name = &self.resized_frames.files[0].file_name;
        let src = self.resized_frames.dir.join(src_file_name);
        let dst = ctx.static_out_path(self.id);
        ctx.bin.magick.convert(src, dst, false).await?;
        info!("converted emote `{:?}` to static sticker", self.id);
        Ok(())
    }
    async fn to_sticker_anim(&self, ctx: &Context, opt: &ConversionOptions) -> Result<()> {
        let output = ctx.anim_out_path(self.id);

        let frames = {
            let base_dir = self.resized_frames.dir.as_path();
            let mut frames = Vec::with_capacity(self.resized_frames.files.len());

            let resized = self.resized_frames.files.iter();
            let durations = self.info.durations.iter();
            for (frame, &duration) in resized.zip(durations) {
                let path = base_dir.join(&frame.file_name);
                frames.push(Img2WebpFrame::new(
                    path,
                    duration,
                    opt.quality,
                    opt.compression_level,
                ))
            }
            frames
        };
        ctx.bin
            .img_2_webp
            .webp_from_images(opt, output, &frames)
            .await?;
        info!("converted emote `{:?}` to animated sticker", self.id);
        Ok(())
    }
    pub async fn to_sticker(&self, ctx: &Context, opt: &ConversionOptions) -> Result<()> {
        if self.info.is_animated() {
            self.to_sticker_anim(ctx, opt).await
        } else {
            self.to_sticker_static(ctx).await
        }
    }
    pub async fn to_sticker_batch(
        ctx: &Context,
        opt: &ConversionOptions,
        emotes: &[Emote],
        par: usize,
    ) -> Result<()> {
        let mut iter = futures::stream::iter(emotes)
            .map(|emote| emote.to_sticker(ctx, opt))
            .buffer_unordered(par);
        while let Some(result) = iter.next().await {
            if let err @ Err(_) = result {
                return err;
            }
        }
        Ok(())
    }

    pub async fn new(ctx: &Context, id: EmoteId) -> Result<Self> {
        // https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
        const ANIMATED_MIN_FRAME_DURATION_MS: i32 = 8;
        const ANIMATED_MAX_TOTAL_DURATION_MS: i32 = 10_000;

        Self::download(ctx, id).await?;

        let info = Self::webp_info(ctx, id).await?;

        if info.is_animated() {
            if info.min_duration().unwrap() < ANIMATED_MIN_FRAME_DURATION_MS {
                return Err(simple_error!("contains too short frames").into());
            } else if info.total_duration() > ANIMATED_MAX_TOTAL_DURATION_MS {
                return Err(simple_error!("exceeds maximum duration").into());
            }
        }

        let raw_frames = Self::extract_frames(ctx, id).await?;

        if info.is_animated() && raw_frames.files.len() != info.frame_count() {
            return Err(simple_error!(
                "frame counts don't match ({} != {})",
                raw_frames.files.len(),
                info.frame_count()
            )
            .into());
        }

        let resized_frames = Self::resize_frames(ctx, id).await?;

        Ok(Self {
            id,
            info,
            raw_frames,
            resized_frames,
        })
    }
    pub async fn new_batch(ctx: &Context, ids: &[EmoteId], par: usize) -> Vec<BatchElement> {
        futures::stream::iter(ids)
            .map(|id| async {
                BatchElement {
                    id: *id,
                    result: Self::new(ctx, *id).await,
                }
            })
            .buffer_unordered(par)
            .collect::<Vec<_>>()
            .await
    }
}
