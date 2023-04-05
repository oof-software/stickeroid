use std::path::{Path, PathBuf};

use anyhow::Result;
use lazy_regex::{lazy_regex, regex_captures};
use log::warn;
use simple_error::simple_error;

pub struct WebpInfo {
    pub durations: Vec<i32>,
    pub size: (i32, i32),
    pub path: PathBuf,
}

impl WebpInfo {
    pub fn from_stdout(stdout: &str, path: impl AsRef<Path>) -> Result<WebpInfo> {
        let mut durations = Vec::new();
        for capture in lazy_regex!(r"^  Duration: (\d+)\r?"m).captures_iter(stdout) {
            let duration_str = capture.get(1).unwrap().as_str();
            durations.push(duration_str.parse::<i32>()?);
        }
        let (_, w, h) = regex_captures!(r"^  Canvas size (\d+) x (\d+)\r?"m, stdout)
            .ok_or(simple_error!("couldn't find webp size"))?;
        let (w, h) = (w.parse::<i32>()?, h.parse::<i32>()?);

        Ok(Self {
            durations,
            size: (w, h),
            path: path.as_ref().to_owned(),
        })
    }
    pub fn is_animated(&self) -> bool {
        !self.durations.is_empty()
    }
    pub fn total_duration(&self) -> i32 {
        self.durations.iter().sum()
    }
    pub fn frame_count(&self) -> usize {
        self.durations.len()
    }
    pub fn all_durations_eq(&self) -> bool {
        let mut iter = self.durations.iter();
        if let Some(first) = iter.next() {
            !iter.any(|duration| duration != first)
        } else {
            true
        }
    }
    pub fn unique_durations(&self) -> Vec<i32> {
        let mut durations = self.durations.clone();
        durations.sort_unstable();
        durations.dedup();
        durations
    }
    pub fn width(&self) -> i32 {
        self.size.0
    }
    pub fn height(&self) -> i32 {
        self.size.1
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn is_valid(&self) -> bool {
        // https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
        const ANIMATED_MIN_FRAME_DURATION_MS: i32 = 8;
        const ANIMATED_MAX_TOTAL_DURATION_MS: i32 = 10_000;

        let mut durations = self.durations.iter();
        if self.total_duration() > ANIMATED_MAX_TOTAL_DURATION_MS {
            warn!("emote `{self:?}` exceeded max duration");
            false
        } else if durations.any(|&d| d < ANIMATED_MIN_FRAME_DURATION_MS) {
            warn!("emote `{self:?}` has frames that are too short");
            false
        } else {
            true
        }
    }
}

impl std::fmt::Debug for WebpInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebpInfo")
            .field("is_animated", &self.is_animated())
            .field("total_duration", &self.total_duration())
            .field("frame_count", &self.frame_count())
            .field("all_durations_eq", &self.all_durations_eq())
            .field("unique_durations", &self.unique_durations())
            .field("size", &self.size)
            .field("path", &self.path())
            .finish()
    }
}
