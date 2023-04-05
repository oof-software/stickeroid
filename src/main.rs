#![allow(dead_code, unreachable_code, unused_variables)]

mod binaries;
mod context;
mod convert;
mod download;
mod emote_ext;
mod file_sequence;
mod list_dir;
mod logging;
mod opt;
mod unwrap_ext;
mod webp;

use context::Context;

use anyhow::Result;

// https://github.com/WhatsApp/stickers/blob/main/Android/app/src/main/java/com/example/samplestickerapp/StickerPackValidator.java#L30-L46
const STATIC_SIZE_LIMIT: usize = 100 * 1024;
const ANIMATED_SIZE_LIMIT: usize = 500 * 1024;
const ANIMATED_MIN_FRAME_DURATION_MS: usize = 8;
const ANIMATED_MAX_TOTAL_DURATION_MS: usize = 10_000;

async fn main_() -> Result<()> {
    logging::init()?;

    let ctx = Context::new()?;
    let _ = ctx.bin.check(2).await?;

    // let ids = ctx.to_emote_ids();
    // ctx.download_emotes(&ids).await?;

    let dls = ctx.downloaded_emotes().await;
    println!("{dls:#?}");

    let infos = ctx.webp_infos(&dls).await?;
    for info in infos {
        println!("{info:?}");
    }

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await.unwrap();
}
