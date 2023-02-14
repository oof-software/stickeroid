#![allow(dead_code)]
mod binaries;
mod file_sequence;
mod logging;
mod opt;
mod webp_frames;

use binaries::{Binaries, FfmpegOptionsBuilder};
use opt::Opt;

use log::{error, info};
use structopt::StructOpt;

fn main() {
    logging::init().unwrap();

    let _opt = Opt::from_args();

    let binaries = match Binaries::from_env() {
        Ok(val) => val,
        Err(err) => {
            error!("{}", err.to_string());
            std::process::exit(1);
        }
    };

    if let Err(err) = binaries.check() {
        error!("{}", err.to_string());
        std::process::exit(1);
    } else {
        info!("checked all needed binaries");
    }

    let info = binaries
        .webp_info
        .run(r"C:\Users\Rico\Pictures\Emotes\emotes\60aee8087e8706b572cf37f3.webp")
        .unwrap();
    println!("{info:?}");

    let opt = FfmpegOptionsBuilder::default()
        .quality(50)
        .compression_level(5)
        .fps(10)
        .build()
        .unwrap();

    binaries
        .ffmpeg
        .webp_from_images(
            r"C:\Users\Rico\Pictures\Emotes\frames\60a62d3aac08622846e7c96f\%04d.png",
            r".\out_test.webp",
            opt,
        )
        .unwrap()
}
