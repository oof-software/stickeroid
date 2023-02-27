use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use crate::binaries::{FfmpegOptions, FfmpegOptionsBuilder};
type ValidatorResult<T> = std::result::Result<T, &'static str>;

pub enum Options {
    Scale(u32, u32),
    Quality(u32),
    CompressionLevel(u32),
    Preset(i32),
    Delay(u32),
    Fps(u32),
    Lossless(bool),
    LoopCount(u32),
}

impl Options {
    fn parse_u32(input: &str, min: i32, max: i32) -> ValidatorResult<i32> {
        let num = input.parse().map_err(|_| "invalid syntax")?;
        if num < min || num > max {
            Err("out of range")
        } else {
            Ok(num)
        }
    }
    fn parse_scale(input: &str) -> ValidatorResult<(u32, u32)> {
        let (w_str, h_str) = input.split_once(':').ok_or("invalid syntax")?;
        let w = Self::parse_u32(w_str, 64, 512)? as u32;
        let h = Self::parse_u32(h_str, 64, 512)? as u32;
        Ok((w, h))
    }

    fn select_scale_blocking() -> Result<Self> {
        let input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Scale (`w:h`)")
            .allow_empty(false)
            .report(false)
            .validate_with(|input: &String| Self::parse_scale(input).map(|_| ()))
            .interact()?;
        let (w, h) = Self::parse_scale(&input).unwrap();
        Ok(Self::Scale(w, h))
    }
    async fn select_scale() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_scale_blocking)
            .await
            .unwrap()
    }
    fn select_quality_blocking() -> Result<Self> {
        let input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Quality")
            .allow_empty(false)
            .report(false)
            .validate_with(|input: &String| Self::parse_u32(input, 0, 100).map(|_| ()))
            .interact()?;
        let quality = Self::parse_u32(&input, 0, 100).unwrap() as u32;
        Ok(Self::Quality(quality))
    }
    async fn select_quality() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_quality_blocking)
            .await
            .unwrap()
    }
    fn select_compression_level_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_compression_level() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_compression_level_blocking)
            .await
            .unwrap()
    }
    fn select_preset_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_preset() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_preset_blocking)
            .await
            .unwrap()
    }
    fn select_delay_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_delay() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_delay_blocking)
            .await
            .unwrap()
    }
    fn select_fps_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_fps() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_fps_blocking)
            .await
            .unwrap()
    }
    fn select_lossless_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_lossless() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_lossless_blocking)
            .await
            .unwrap()
    }
    fn select_loop_count_blocking() -> Result<Self> {
        unimplemented!()
    }
    async fn select_loop_count() -> Result<Self> {
        tokio::task::spawn_blocking(Self::select_loop_count_blocking)
            .await
            .unwrap()
    }

    fn apply(self, opt: &mut FfmpegOptionsBuilder) {
        match self {
            Options::Scale(w, h) => opt.scale((w, h)),
            Options::Quality(q) => opt.quality(q),
            Options::CompressionLevel(m) => opt.compression_level(m),
            Options::Preset(preset) => opt.preset(preset),
            Options::Delay(ms) => opt.delay_ms(ms),
            Options::Fps(fps) => opt.fps(fps),
            Options::Lossless(lossless) => opt.lossless(lossless),
            Options::LoopCount(loop_count) => opt.loop_count(loop_count),
        };
    }
    pub async fn loop_select() -> Result<FfmpegOptions> {
        const OPTIONS: [&str; 9] = [
            "Scale",
            "Quality",
            "Compression Level",
            "Preset",
            "Delay",
            "FPS",
            "Lossless",
            "Loop Count",
            "Finish",
        ];

        let mut builder = FfmpegOptions::builder();
        loop {
            let index = tokio::task::spawn_blocking(move || {
                Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("What do you want to change?")
                    .default(0)
                    .report(false)
                    .items(&OPTIONS)
                    .interact()
            })
            .await?
            .unwrap();

            let thing = match index {
                0 /* Scale */ => Self::select_scale().await?,
                1 /* Quality */ => Self::select_quality().await?,
                2 /* Compression Level */ => Self::select_compression_level().await?,
                3 /* Preset */ => Self::select_preset().await?,
                4 /* Delay */ => Self::select_delay().await?,
                5 /* FPS */ => Self::select_fps().await?,
                6 /* Lossless */ => Self::select_lossless().await?,
                7 /* Loop Count */ => Self::select_loop_count().await?,
                _ /* Finish */ => break,
            };
            thing.apply(&mut builder);
        }
        Ok(builder.build()?)
    }
}
