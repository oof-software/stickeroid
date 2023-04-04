use anyhow::Result;
use lazy_regex::lazy_regex;

pub struct WebpFrames {
    pub durations: Vec<u32>,
}

impl WebpFrames {
    pub fn from_webp_info(stdout: &str) -> Result<WebpFrames> {
        let mut durations = Vec::new();
        for capture in lazy_regex!(r"^  Duration: (\d+)\r?"m).captures_iter(stdout) {
            let duration_str = capture.get(1).unwrap().as_str();
            durations.push(duration_str.parse::<u32>()?);
        }
        Ok(Self { durations })
    }
    pub fn is_animated(&self) -> bool {
        !self.durations.is_empty()
    }
    pub fn total_duration(&self) -> u32 {
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
}
impl std::fmt::Debug for WebpFrames {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebpFrames")
            .field("is_animated", &self.is_animated())
            .field("total_duration", &self.total_duration())
            .field("frame_count", &self.frame_count())
            .field("all_durations_eq", &self.all_durations_eq())
            .finish_non_exhaustive()
    }
}
