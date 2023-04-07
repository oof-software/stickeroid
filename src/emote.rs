use anyhow::Result;
use futures::StreamExt;
use log::{info, warn};
use simple_error::simple_error;

use crate::binaries::Img2WebpFrame;
use crate::context::Context;
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
    async fn to_sticker_anim(&self, ctx: &Context) -> Result<()> {
        #[derive(Debug, Clone, Copy)]
        struct Preset {
            pub quality: i32,
            pub level: i32,
        }
        impl Preset {
            pub const fn new(quality: i32, level: i32) -> Preset {
                Preset { quality, level }
            }
        }

        // https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
        const STATIC_SIZE_LIMIT: u64 = 100 * 1024;
        const ANIMATED_SIZE_LIMIT: u64 = 500 * 1024;

        const QUALITY_PRESETS: [Preset; 5] = [
            Preset::new(75, 4),
            Preset::new(50, 4),
            Preset::new(25, 4),
            Preset::new(10, 4),
            Preset::new(10, 6),
        ];

        fn make_frames(resized_frames: &FileSequence, durations: &[i32]) -> Vec<Img2WebpFrame> {
            let base_dir = resized_frames.dir.as_path();
            let mut frames = Vec::with_capacity(resized_frames.files.len());
            let resized = resized_frames.files.iter();
            for (frame, &duration) in resized.zip(durations.iter()) {
                let path = base_dir.join(&frame.file_name);
                frames.push(Img2WebpFrame::new(path, duration, 0, 0))
            }
            frames
        }
        fn alter_frames(frames: &mut [Img2WebpFrame], preset: Preset) {
            frames.iter_mut().for_each(|frame| {
                frame.compression_quality = preset.quality;
                frame.compression_method = preset.level;
            });
        }

        let output = ctx.anim_out_path(self.id);
        let mut frames = make_frames(&self.resized_frames, &self.info.durations);

        let mut success = false;
        for preset in QUALITY_PRESETS {
            alter_frames(&mut frames, preset);

            ctx.bin
                .img_2_webp
                .webp_from_images(&output, &frames)
                .await?;

            if crate::fs::file_size(&output).await? > ANIMATED_SIZE_LIMIT {
                warn!("emote `{:?}` too large with {:?}", self.id, preset);
            } else {
                info!(
                    "converted emote `{:?}` to animated sticker with {:?}",
                    self.id, preset
                );
                success = true;
                break;
            }
        }

        if !success {
            warn!("emote `{:?}` too large with every preset", self.id);
        }

        Ok(())
    }
    pub async fn to_sticker(&self, ctx: &Context) -> Result<()> {
        if self.info.is_animated() {
            self.to_sticker_anim(ctx).await
        } else {
            self.to_sticker_static(ctx).await
        }
    }
    pub async fn to_sticker_batch(ctx: &Context, emotes: &[Emote], par: usize) -> Result<()> {
        let mut iter = futures::stream::iter(emotes)
            .map(|emote| emote.to_sticker(ctx))
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
