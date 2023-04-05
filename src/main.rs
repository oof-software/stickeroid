#![allow(dead_code, unreachable_code, unused_variables)]
#![feature(iter_partition_in_place)]

mod binaries;
mod context;
mod convert;
mod download;
mod emote_ext;
mod file_sequence;
mod fs;
mod list_dir;
mod logging;
mod opt;
mod unwrap_ext;
mod webp;

use context::Context;

use anyhow::Result;

async fn main_() -> Result<()> {
    logging::init()?;

    let ctx = Context::new()?;
    let _ = ctx.bin.check(3).await?;

    let ids = ctx.to_emote_ids();
    ctx.download_emotes(&ids).await?;

    let mut infos = ctx.webp_infos(&ids).await?;
    let (valid, _) = Context::partition_infos_valid(&mut infos);

    ctx.extract_frames(&ids).await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await.unwrap();
}
