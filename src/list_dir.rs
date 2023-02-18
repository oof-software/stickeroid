use std::ffi::OsStr;
use std::path::Path;

use log::warn;
use walkdir::{DirEntry, WalkDir};

fn has_ext<P>(entry: &DirEntry, ext: P) -> bool
where
    P: AsRef<OsStr>,
{
    entry
        .path()
        .extension()
        .map(|ext_| ext_ == ext.as_ref())
        .unwrap_or(false)
}
fn is_file(entry: &DirEntry) -> bool {
    entry.metadata().map_or(false, |meta| meta.is_dir())
}

/// Collects any file or folder with an extension by
/// [`Path::extension`](std::path::Path::extension) non recursive.
pub fn files_with_ext_blocking<P, Q>(path: P, ext: Q) -> Vec<DirEntry>
where
    P: AsRef<Path>,
    Q: AsRef<OsStr>,
{
    let walk = WalkDir::new(path.as_ref()).max_depth(1).min_depth(1);
    let buffer = walk
        .into_iter()
        .filter_entry(|entry| has_ext(entry, ext.as_ref()) && is_file(entry))
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();

    if buffer.is_empty() {
        warn!("no files matched `file_with_ext` query");
    }

    buffer
}

pub async fn files_with_ext<P, Q>(path: P, ext: Q) -> Vec<DirEntry>
where
    P: AsRef<Path>,
    Q: AsRef<OsStr>,
{
    let path = path.as_ref().to_owned();
    let ext = ext.as_ref().to_owned();
    tokio::task::spawn_blocking(move || files_with_ext_blocking(path, ext))
        .await
        .unwrap()
}
