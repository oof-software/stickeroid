#![allow(dead_code)]
mod binaries;
mod bttv;
mod download;
mod file_sequence;
mod logging;
mod opt;
mod seven_tv;
mod webp_frames;

use binaries::Binaries;
use opt::Opt;

use log::{error, info};
use structopt::StructOpt;

use crate::seven_tv::{ids_from_file, seven_tv_emotes};

async fn main_() {
    logging::init().unwrap();

    let _opt = Opt::from_args();

    let binaries = match Binaries::from_env() {
        Ok(val) => val,
        Err(err) => {
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    if let Err(err) = binaries.check().await {
        error!("{}", err.to_string());
        std::process::exit(1);
    } else {
        info!("checked all needed binaries");
    }

    let client = download::client().unwrap();

    let emotes = ids_from_file("./emotes.txt").await.unwrap();
    let downloads = seven_tv_emotes(&client, emotes.iter(), 3)
        .await
        .into_iter()
        .filter_map(|d| d.ok())
        .collect::<Vec<_>>();

    info!(
        "downloaded {} out of {} files",
        downloads.len(),
        emotes.len()
    );

    std::fs::create_dir("./avif/").unwrap();
    for file in downloads.iter() {
        file.save_to_file("./avif/").await.unwrap();
        info!(
            "saved `{}` to disk",
            file.file_name.to_str().unwrap_or_default()
        );
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    main_().await;
}
