use std::path::{Path, PathBuf};

use anyhow::Result;
use lazy_regex::regex_captures;
use thiserror::Error;
use walkdir::{DirEntry, WalkDir};

pub struct SequenceElement {
    pub index: usize,
    pub path: PathBuf,
}

#[derive(Debug, Error)]
pub enum SequenceError {
    #[error("file name `{0}` does not match the regex")]
    NoMatch(String),
    #[error("file name `{0}` is an invalid sequence")]
    Parse(String),
    #[error("file names in `{0}` don't form a valid sequence")]
    Sequence(String),
    #[error("no files found in `{0}`")]
    Empty(String),
}

impl TryFrom<DirEntry> for SequenceElement {
    type Error = SequenceError;
    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        let file_name = value.file_name().to_str().unwrap();
        let (_, digits) = regex_captures!(r"^(\d+)\.\w{3,4}$", file_name)
            .ok_or(SequenceError::NoMatch(file_name.to_string()))?;
        let index = digits
            .parse::<usize>()
            .map_err(|_| SequenceError::Parse(file_name.to_string()))?;
        let path = value.into_path();
        Ok(Self { index, path })
    }
}

/// Collects any file or folder that matches `^(\d+)\.\w{3,4}$` within the given `path`
/// where the group `(\d+)` denotes the sequence index.
///
/// E.g. `0001.png` or `002.webp`
pub fn file_sequence_blocking<P>(path: P) -> Result<Vec<SequenceElement>>
where
    P: AsRef<Path>,
{
    let mut buffer = WalkDir::new(path.as_ref())
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(SequenceElement::try_from)
        .collect::<std::result::Result<Vec<_>, _>>()?;

    buffer.sort_by_key(|e| e.index);

    if !buffer.is_empty() {
        let expected_len = buffer.last().unwrap().index - buffer.first().unwrap().index + 1;
        if buffer.len() != expected_len {
            let path = path.as_ref().to_str().unwrap().to_string();
            Err(SequenceError::Sequence(path).into())
        } else {
            Ok(buffer)
        }
    } else {
        let path = path.as_ref().to_str().unwrap().to_string();
        Err(SequenceError::Empty(path).into())
    }
}

pub async fn file_sequence<P>(path: P) -> Result<Vec<SequenceElement>>
where
    P: AsRef<Path>,
{
    let path = path.as_ref().to_owned();
    tokio::task::spawn_blocking(move || file_sequence_blocking(path))
        .await
        .unwrap()
}
