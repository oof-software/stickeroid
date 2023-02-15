#![allow(dead_code)]
mod binaries;
mod download;
mod file_sequence;
mod logging;
mod opt;
mod webp_frames;

use binaries::Binaries;
use opt::Opt;

use log::{error, info};
use structopt::StructOpt;

use crate::download::{seven_tv_emotes, seven_tv_ids_from_file};

#[tokio::main(flavor = "current_thread")]
async fn main() {
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
    let emotes = seven_tv_ids_from_file("./emotes.txt").await.unwrap();
    let downloads = seven_tv_emotes(&client, emotes.iter(), 3).await;
    let successes = downloads.iter().filter(|d| d.is_ok()).count();

    info!("downloaded {} out of {} files", successes, emotes.len());
}
