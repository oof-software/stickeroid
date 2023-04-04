#![allow(dead_code, unreachable_code, unused_variables)]

mod binaries;
mod binaries_ext;
mod bttv;
mod download;
mod emote_process;
mod file_sequence;
mod list_dir;
mod logging;
mod opt;
mod pipeline;
mod seven_tv;
mod webp_frames;

use std::path::{Path, PathBuf};
use std::str::FromStr;

use binaries::{Binaries, FfmpegOptions};
use emote_process::make_ffmpeg_options;
use futures::StreamExt;
use list_dir::files_with_ext;
use opt::Opt;

use anyhow::Result;
use log::{error, info, warn};
use structopt::StructOpt;
use walkdir::DirEntry;

use crate::seven_tv::{ids_from_file, seven_tv_emotes};

const DL_PATH: &str = "./dl/";
const FRAMES_PATH: &str = "./frames/";
const OUT_PATH: &str = "./out/";

// https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
const STATIC_SIZE_LIMIT: usize = 100 * 1024;
const ANIMATED_SIZE_LIMIT: usize = 500 * 1024;
const ANIMATED_MIN_FRAME_DURATION_MS: usize = 8;
const ANIMATED_MAX_TOTAL_DURATION_MS: usize = 10_000;

async fn download_emotes_to_dir<P>(emotes: &[String], path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let client = download::client()?;

    let downloads = seven_tv_emotes(&client, emotes, 5).await;
    let successes = downloads
        .into_iter()
        .filter_map(|d| d.ok())
        .collect::<Vec<_>>();

    let path_str = path.as_ref().to_str().unwrap_or_default();
    if let Err(err) = tokio::fs::create_dir(path.as_ref()).await {
        warn!("couldn't create `{path_str}`: {err}",);
    }

    // TODO: Don't keep all bytes in memory at once
    for file in successes.iter() {
        file.save_to_dir(path.as_ref()).await?;
    }
    Ok(())
}

async fn extract_emote_frames<P>(binaries: &Binaries, emote: &DirEntry, dst_dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if let Err(err) = tokio::fs::create_dir(&dst_dir).await {
        warn!("couldn't create `{}`: {}", dst_dir.as_ref().display(), err);
    }
    binaries
        .anim_dump
        .dump_frames(emote.path(), dst_dir.as_ref())
        .await
}

async fn process_emote(binaries: &Binaries, ffmpeg_opts: &FfmpegOptions, emote_file: &DirEntry) {
    let file_name = emote_file
        .file_name()
        .to_str()
        .expect("not a fucked filename");
    let id = file_name.split_once('.').unwrap().0;
    let dst_dir = PathBuf::from_str(&format!("./frames/{id}/")).unwrap();

    if let Ok(meta) = tokio::fs::metadata(&dst_dir).await {
        if !meta.is_dir() {
            warn!(
                "frame directory exists but is not a dir `{}`",
                dst_dir.display()
            );
            return;
        } else {
            info!(
                "frame directory already exists, delete to reextract `{}`",
                dst_dir.display()
            );
            return;
        }
    }

    match extract_emote_frames(binaries, emote_file, &dst_dir).await {
        Ok(_) => info!("extracted frames to `{}`", dst_dir.display()),
        Err(err) => info!(
            "couldn't extract frames to `{}`: {}",
            dst_dir.display(),
            err
        ),
    };
}

async fn main_() -> Result<()> {
    logging::init()?;

    let _opt = Opt::from_args();

    let binaries = match Binaries::from_env() {
        Ok(val) => {
            if let Err(err) = val.check(3).await {
                error!("{}", err.to_string());
                std::process::exit(1);
            } else {
                info!("checked all needed binaries");
                val
            }
        }
        Err(err) => {
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    let ffmpeg_opts = make_ffmpeg_options().await;

    info!("loading emote ids from `./ids_7tv.txt`");
    let emotes = ids_from_file("./ids_7tv.txt").await?;

    info!("downloading emotes from 7tv");
    // download_emotes_to_dir(&emotes, "./dl/").await?;

    info!("collecting downloaded emote files");
    let emote_files = files_with_ext("./dl/", ".webp").await;

    info!("star processing emotes");
    futures::stream::iter(emote_files.iter())
        .map(|emote_file| process_emote(&binaries, &ffmpeg_opts, emote_file))
        .buffer_unordered(5)
        .collect::<Vec<()>>()
        .await;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await.unwrap();
}
