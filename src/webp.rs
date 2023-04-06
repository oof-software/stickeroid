use anyhow::Result;
use lazy_regex::{lazy_regex, regex_captures};
use simple_error::simple_error;

#[derive(Clone)]
pub struct WebpInfo {
    pub durations: Vec<i32>,
    pub size: (i32, i32),
}

impl WebpInfo {
    pub fn from_stdout(stdout: &str) -> Result<WebpInfo> {
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
    pub fn min_duration(&self) -> Option<i32> {
        self.durations.iter().min().copied()
    }
    pub fn max_duration(&self) -> Option<i32> {
        self.durations.iter().max().copied()
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
            .finish()
    }
}
