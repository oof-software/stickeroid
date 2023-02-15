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

use crate::download::seven_tv_emotes;

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

    if let Err(err) = binaries.check() {
        error!("{}", err.to_string());
        std::process::exit(1);
    } else {
        info!("checked all needed binaries");
    }

    let emotes = [
        "60ae958e229664e8667aea38",
        "60ae7316f7c927fad14e6ca2",
        "60aea4074b1ea4526d3c97a9",
        "60a1babb3c3362f9a4b8b33a",
        "60abf171870d317bef23d399",
        "60ae65b29627f9aff4fd8bef",
        "60ae4ec30e35477634988c18",
        "603cac391cd55c0014d989be",
        "60a95f109d598ea72fad13bd",
        "60ae8d9ff39a7552b658b60d",
        "60ae4bb30e35477634610fda",
        "60aec2196cfcffe15f4e4f93",
        "638767f24cc489ef45239272",
        "60ccf4479f5edeff9938fa77",
        "60aee9d5361b0164e60d02c2",
        "60a487509485e7cf2f5a6fa7",
        "60aeec1712d7701491f89cf5",
        "629f7ed33cfb54ec859bb216",
        "60a9cfe96daf811370b0b640",
        "6042089e77137b000de9e669",
        "60afc2eaa3648f409a82e80b",
        "6102a37ba57eeb23c0e3e5cb",
        "6040a8bccf6746000db10348",
        "60afbe0599923bbe7fe9bae1",
        "60ae3e54259ac5a73e56a426",
        "60aed440997b4b396ed9ec39",
        "60b14a737a157a7f3360fb32",
        "60e0ec549db74f240c4c0c5b",
        "60b00d1f0d3a78a196f803e3",
        "60b0c36388e8246a4b120d7e",
        "611523959bf574f1fded6d72",
        "60af03597e8706b57220e8ce",
        "604097c3cf6746000db10344",
        "60af769d2c36aae19e32ec9d",
        "60a58e67a71d9fd11049f5e9",
        "6145e8b10969108b671957ec",
        "603cb5e1c20d020014423c68",
        "60be91ac412138e6fa80284d",
        "60bcb44f7229037ee386d1ab",
        "60b056f5b254a5e16b929707",
        "60aed4fe423a803ccae373d3",
        "60ae6a7b117ec68ca434404e",
        "60aeeb53a564afa26ee82323",
        "60aef3e4b74ea8ff797ae5ac",
        "60af9ee712f90fadd6d75af3",
        "60aea1dbf39a7552b6ccb61d",
        "60ae2376b2ecb015058f4aa7",
        "60af0382b38361ea91337096",
        "604a93564d948c001460998b",
        "603eaaa9115b55000d7282d8",
        "612e638afc02cc1a1f411b2d",
        "60aef388b38361ea914aad89",
        "60aed217c9cf495e5be86812",
        "60b2876f4f32610f15bfc5dc",
        "60b0da234daf0d3e21e3755d",
        "603caea243b9e100141caf4f",
        "60aede8f12d770149120019d",
        "60ba5b527955f57f43179793",
        "603caf09c20d020014423c14",
        "60420a8b77137b000de9e66e",
        "60b01e91ad7fb4b50b3a3eaf",
        "60a304efac2bcb20ef20fa89",
        "603eace1115b55000d7282db",
        "60aef3aea564afa26e686d8c",
        "60aecb385174a619dbc175be",
        "6266ded14f54759b7184de2d",
        "60a5c7bec2ca47c7d5da99e1",
        "6141f07a962a60904864895e",
        "613937fcf7977b64f644c0d2",
        "63438a743d1bc89e0ff9e400",
        "60ae9173f39a7552b68f9730",
        "603caee4c20d020014423c13",
        "618330c5f1ae15abc7ebb8c6",
        "603cad8f16b3f90014d31858",
        "60eefddf2c24e9e0e6ec9141",
        "60af9e3b52a13d1adb78e15e",
        "618302fe8d50b5f26ee7b9bc",
        "634493ce05c2b2cd864d5f0d",
        "60ae3258259ac5a73e013302",
        "60ef410f48cde2fcc3eb5caa",
        "62f424b0ea941a22a1f03268",
        "60af1ba684a2b8e655387bba",
        "60af990d566c3e1fc9d26c93",
        "60b0ee63f12983cd1d00c565",
        "60ae387cb2ecb0150505e235",
        "633aa7026a33c6c1976d8e2b",
        "60a5e53635a601f90117c797",
        "60845a505e01df61570a6f1d",
        "60aef48211a994a4ac3e00db",
        "60bd742760b022372c872ef1",
    ];

    let client = download::client().unwrap();

    let downloads = seven_tv_emotes(&client, emotes, 3).await;
    let successes = downloads.iter().filter(|d| d.is_ok()).count();

    info!("downloaded {} out of {} files", successes, emotes.len());
}
