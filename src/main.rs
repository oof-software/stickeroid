#[allow(dead_code)]
mod binaries;
mod logging;
mod opt;
mod webp_frames;

use binaries::Binaries;
use opt::Opt;

use log::{error, info};
use structopt::StructOpt;

fn main() {
    logging::init().unwrap();

    let opt = Opt::from_args();

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
}