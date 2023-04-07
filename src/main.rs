#![allow(dead_code, unreachable_code, unused_variables)]

mod binaries;
mod context;
mod convert;
mod download;
mod emote;
mod emote_ext;
mod file_sequence;
mod fs;
mod list_dir;
mod logging;
mod opt;
mod unwrap_ext;
mod webp;

use crate::context::Context;
use crate::emote::Emote;

use anyhow::Result;
use log::warn;

async fn main_() -> Result<()> {
    logging::init()?;

    let ctx = Context::new()?;
    let _ = ctx.bin.check(3).await?;

    let ids = ctx.to_emote_ids();

    let processed = Emote::new_batch(&ctx, &ids, 5)
        .await
        .into_iter()
        .filter_map(|batch| {
            if let Err(e) = &batch.result {
                warn!("couldn't process `{:?}`: {}", batch.id, e);
                None
            } else {
                Some(batch.result.unwrap())
            }
        })
        .collect::<Vec<_>>();

    Emote::to_sticker_batch(&ctx, &processed, 14).await?;

    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await.unwrap();
}
