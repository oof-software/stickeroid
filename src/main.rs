#![allow(dead_code, unreachable_code)]

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

use binaries::Binaries;
use emote_process::Options;
use futures::StreamExt;
use list_dir::files_with_ext;
use opt::Opt;

use anyhow::Result;
use log::{error, info, warn};
use structopt::StructOpt;
use walkdir::DirEntry;

use crate::seven_tv::seven_tv_emotes;

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

    let options = Options::loop_select().await.unwrap();
    println!("{options:#?}");

    std::process::exit(1);

    // let emotes = ids_from_file("./ids_7tv.txt").await?;
    // download_emotes_to_dir(&emotes, "./avif/").await?;

    let emote_files = files_with_ext("./avif/", "webp").await;
    futures::stream::iter(emote_files.iter())
        .map(|emote_file| {
            let binaries = &binaries;
            async move {
                let file_name = match emote_file.file_name().to_str() {
                    Some(v) => v.to_string(),
                    None => {
                        warn!("invalid file_name `{}`", emote_file.path().display());
                        return;
                    }
                };

                let dst_dir = PathBuf::from_str(&format!("./avif/_{file_name}")).unwrap();
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
        })
        .buffer_unordered(5)
        .collect::<Vec<()>>()
        .await;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await.unwrap();
}
