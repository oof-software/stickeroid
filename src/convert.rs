use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};

use crate::binaries::{FfmpegOptions, FfmpegOptionsBuilder};
type ValidatorResult<T> = std::result::Result<T, &'static str>;

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
    fn parse_u32(input: &str, min: i32, max: i32) -> ValidatorResult<i32> {
        let num = input.parse().map_err(|_| "invalid syntax")?;
        if num < min || num > max {
            Err("out of range")
        } else {
            Ok(num)
        }
    }
    fn parse_scale(input: &str) -> ValidatorResult<(i32, i32)> {
        let parser = Self::parse_in_range(64, 512);
        let (w_str, h_str) = input.split_once(':').ok_or("invalid syntax")?;
        Ok((parser(w_str)?, parser(h_str)?))
    }
    fn parse_in_range(min: i32, max: i32) -> impl Fn(&str) -> ValidatorResult<i32> {
        move |input: &str| -> ValidatorResult<i32> {
            let val = input.parse::<i32>().map_err(|_| "invalid_syntax")?;
            if val < min || val > max {
                Err("value out of range")
            } else {
                Ok(val)
            }
        }
    }
    fn prompt_text<F>(prompt: &str, validator: F) -> String
    where
        F: FnMut(&String) -> ValidatorResult<()>,
    {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .allow_empty(false)
            .report(false)
            .validate_with(validator)
            .interact()
            .unwrap()
    }
    fn prompt_select(prompt: &str, items: &[&str]) -> usize {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(items)
            .report(false)
            .interact()
            .unwrap()
    }
    fn prompt_select_opt(prompt: &str, items: &[&str]) -> Option<usize> {
        Select::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(0)
            .items(items)
            .report(false)
            .interact_opt()
            .unwrap()
    }

    async fn select_scale() -> Self {
        tokio::task::spawn_blocking(|| {
            let input = Self::prompt_text("Scale (`w:h`)", |input| {
                Self::parse_scale(input).map(|_| ())
            });
            let (w, h) = Self::parse_scale(&input).unwrap();
            Self::Scale(w, h)
        })
        .await
        .unwrap()
    }
    async fn select_quality() -> Self {
        tokio::task::spawn_blocking(|| {
            let parser = Self::parse_in_range(0, 100);
            let input = Self::prompt_text("Quality", |input| parser(input).map(|_| ()));
            Self::Quality(parser(&input).unwrap())
        })
        .await
        .unwrap()
    }
    async fn select_compression_level() -> Self {
        tokio::task::spawn_blocking(|| {
            let parser = Self::parse_in_range(0, 100);
            let input = Self::prompt_text("Compression Level", |input| parser(input).map(|_| ()));
            Self::CompressionLevel(parser(&input).unwrap())
        })
        .await
        .unwrap()
    }
    async fn select_preset() -> Self {
        const ITEMS: [&str; 7] = [
            "None", "Default", "Picture", "Photo", "Drawing", "Icon", "Text",
        ];

        tokio::task::spawn_blocking(|| {
            let index = Self::prompt_select("Preset", &ITEMS);
            Self::Preset(index as i32 - 1)
        })
        .await
        .unwrap()
    }
    async fn select_delay() -> Self {
        tokio::task::spawn_blocking(|| {
            let parser = Self::parse_in_range(5, 1000);
            let input = Self::prompt_text("Delay (ms)", |input| parser(input).map(|_| ()));
            Self::Delay(parser(&input).unwrap())
        })
        .await
        .unwrap()
    }
    async fn select_fps() -> Self {
        tokio::task::spawn_blocking(|| {
            let parser = Self::parse_in_range(1, 60);
            let input = Self::prompt_text("FPS", |input| parser(input).map(|_| ()));
            Self::Fps(parser(&input).unwrap())
        })
        .await
        .unwrap()
    }
    async fn select_lossless() -> Self {
        const ITEMS: [&str; 2] = ["Yes", "No"];

        tokio::task::spawn_blocking(|| {
            let index = Self::prompt_select("Lossless", &ITEMS);
            Self::Lossless(index as i32)
        })
        .await
        .unwrap()
    }
    async fn select_loop_count() -> Self {
        tokio::task::spawn_blocking(|| {
            let parser = Self::parse_in_range(0, 100);
            let input = Self::prompt_text("Loop-Count", |input| parser(input).map(|_| ()));
            Self::LoopCount(parser(&input).unwrap())
        })
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
    async fn select() -> Option<Self> {
        const ITEMS: [&str; 7] = [
            "Change Quality",
            "Change Compression-Level",
            "Change Preset",
            "Change Delay",
            "Change FPS",
            "Change Lossless",
            "Change Loop-Count",
        ];

        let index =
            tokio::task::spawn_blocking(|| Self::prompt_select_opt("What to Change", &ITEMS))
                .await
                .unwrap()?;

        match index {
            0 => Some(Self::select_quality().await),
            1 => Some(Self::select_compression_level().await),
            2 => Some(Self::select_preset().await),
            3 => Some(Self::select_delay().await),
            4 => Some(Self::select_fps().await),
            5 => Some(Self::select_lossless().await),
            6 => Some(Self::select_loop_count().await),
            _ => unreachable!(),
        }
    }
}

pub async fn make_ffmpeg_options() -> FfmpegOptions {
    let mut builder = FfmpegOptions::builder();

    loop {
        match Options::select().await {
            None => break,
            Some(opt) => opt.apply(&mut builder),
        };
    }

    // SAFETY: Every field has a default
    builder.build().unwrap()
}
