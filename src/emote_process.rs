use std::ops::ControlFlow;

use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use lazy_static::lazy_static;
use log::info;
use regex::Regex;

use crate::binaries::FfmpegOptions;
type ValidatorResult<T> = std::result::Result<T, &'static str>;

pub fn test_ui_blocking() -> Result<()> {
    let selections = [
        "60898a7739b5010444d07e6e",
        "6088b8f839b5010444d078d4",
        "6027ea208fbb823604bde323",
        "59a4ea2865231102cde26e9c",
        "60b13dfcf8b3f62601c34b9f",
        "5805580c3d506fea7ee357d6",
    ];

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an emote to process")
        .default(0)
        .items(&selections)
        .report(false)
        .interact()?;
    let selected = selections[selected_index];

    info!("Selected `{selected}`");

    Ok(())
}

pub async fn test_ui() -> Result<()> {
    tokio::task::spawn_blocking(|| test_ui_blocking())
        .await
        .unwrap()
}

pub enum Options {
    Scale(i32, i32),
    Quality(i32),
    CompressionLevel(i32),
    Preset(i32),
    Delay(i32),
    Fps(i32),
    Lossless(i32),
    LoopCount(i32),
}

impl Options {
    fn names() -> Vec<&'static str> {
        Vec::from_iter([
            "Scale",
            "Quality",
            "Compression Level",
            "Preset",
            "Delay",
            "FPS",
            "Lossless",
            "Loop Count",
        ])
    }

    fn parse_scale(input: &str) -> ValidatorResult<(i32, i32)> {
        let (w_str, h_str) = input.split_once(':').ok_or("invalid syntax")?;
        let w = w_str.parse::<u32>().map_err(|_| "invalid syntax")? as i32;
        let h = h_str.parse::<u32>().map_err(|_| "invalid syntax")? as i32;
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
        tokio::task::spawn_blocking(|| Self::select_scale_blocking())
            .await
            .unwrap()
    }
    async fn select_quality() -> Self {
        unimplemented!()
    }
    async fn select_compression_level() -> Self {
        unimplemented!()
    }
    async fn select_preset() -> Self {
        unimplemented!()
    }
    async fn select_delay() -> Self {
        unimplemented!()
    }
    async fn select_fps() -> Self {
        unimplemented!()
    }
    async fn select_lossless() -> Self {
        unimplemented!()
    }
    async fn select_loop_count() -> Self {
        unimplemented!()
    }

    fn apply(&self, opt: &mut FfmpegOptions) {
        match self {
            Options::Scale(w, h) => todo!(),
            Options::Quality(q) => todo!(),
            Options::CompressionLevel(m) => todo!(),
            Options::Preset(preset) => todo!(),
            Options::Delay(ms) => todo!(),
            Options::Fps(fps) => todo!(),
            Options::Lossless(lossless) => todo!(),
            Options::LoopCount(loop_count) => todo!(),
        }
    }
    async fn select(opt: &mut FfmpegOptions) -> ControlFlow<()> {
        unimplemented!()
    }
}

pub async fn make_ffmpeg_options() -> Result<FfmpegOptions> {
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

    pub fn inner_blocking() -> std::io::Result<usize> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Change options for FFmpeg")
            .default(0)
            .report(false)
            .items(&OPTIONS)
            .interact()
    }
    pub async fn inner() -> std::io::Result<usize> {
        tokio::task::spawn_blocking(|| inner_blocking())
            .await
            .unwrap()
    }

    let mut builder = FfmpegOptions::builder();
    loop {
        let selected_index = inner().await?;
        match selected_index {
            0 => info!("Changing Scale"),
            1 => info!("Changing Quality"),
            2 => info!("Changing Compression Level"),
            3 => info!("Changing Preset"),
            4 => info!("Changing Delay"),
            5 => info!("Changing FPS"),
            6 => info!("Changing Lossless"),
            7 => info!("Changing Loop Count"),
            _ => break,
        }
    }
    let options = builder.build()?;
    Ok(options)
}
