use std::path::Path;

use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

fn matches_regex(entry: &DirEntry) -> Option<u32> {
    lazy_static! {
        static ref SEQUENCE_RE: Regex = Regex::new(r"^(\d+)\.\w{3,4}$").unwrap();
    }

    let file_name = entry.file_name().to_str()?;
    let digits = SEQUENCE_RE.captures(file_name)?.get(1).unwrap().as_str();
    digits.parse().ok()
}

/// Collects any file or folder that matches `^(\d+)\.\w{3,4}$` within the given `path`
/// where the group `(\d+)` denotes the sequence index.
///
/// E.g. `0001.png` or `002.webp`
pub fn file_sequence_blocking<P>(path: P) -> Vec<(u32, DirEntry)>
where
    P: AsRef<Path>,
{
    let walk = WalkDir::new(path.as_ref()).max_depth(1).min_depth(1);
    let mut buffer = walk
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| Some((matches_regex(&entry)?, entry)))
        .collect::<Vec<_>>();
    buffer.sort_by_key(|e| e.0);

    if buffer.is_empty() {
        warn!("no files matched the file_sequence query");
    } else {
        let first = buffer.first().unwrap().0 as usize;
        let last = buffer.last().unwrap().0 as usize;
        if buffer.len() != last - first + 1 {
            warn!("numbers in filenames are inconsistent");
        }
    }

    buffer
}

pub async fn file_sequence<P>(path: P) -> Vec<(u32, DirEntry)>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_owned();
    tokio::task::spawn_blocking(move || file_sequence_blocking(path))
        .await
        .unwrap()
}
