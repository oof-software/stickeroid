use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirPathParseError {
    #[error("couldn't create directory: {0}")]
    Create(std::io::Error),
    #[error("doesn't correspond to a directory")]
    InvalidType,
}

#[derive(Error, Debug)]
pub enum FilePathParseError {
    #[error("couldn't fetch metadata: {0}")]
    NoMetadata(#[from] std::io::Error),
    #[error("doesn't correspond to a file")]
    InvalidType,
}

fn parse_dir_path(src: &str) -> Result<PathBuf, DirPathParseError> {
    let path = PathBuf::from(src);
    match path.metadata() {
        Ok(meta) => {
            if !meta.is_dir() {
                Err(DirPathParseError::InvalidType)
            } else {
                Ok(path)
            }
        }
        Err(_) => {
            if let Err(err) = std::fs::create_dir(&path) {
                Err(DirPathParseError::Create(err))
            } else {
                Ok(path)
            }
        }
    }
}

fn parse_file_path(src: &str) -> Result<PathBuf, FilePathParseError> {
    let path = PathBuf::from(src);
    let meta = path.metadata()?;
    if !meta.is_file() {
        Err(FilePathParseError::InvalidType)
    } else {
        Ok(path)
    }
}

#[derive(Error, Debug)]
pub enum IdsParseError {
    #[error("some ids are invalid")]
    InvalidIds,
    #[error("couldn't open file")]
    InvalidFile,
    #[error("file contains invalid ids")]
    InvalidIdsInFile,
}

fn parse_id_file(src: &str) -> Result<Vec<String>, IdsParseError> {
    lazy_static! {
        static ref ID_RE: Regex = RegexBuilder::new(r"^([a-f0-9]{24})\r?$")
            .multi_line(true)
            .build()
            .unwrap();
    }

    let data = std::fs::read_to_string(src).map_err(|_| IdsParseError::InvalidFile)?;
    Ok(ID_RE
        .captures_iter(&data)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect::<Vec<_>>())
}

#[derive(Debug, StructOpt)]
#[structopt(name = "convertoid", about = "Convert stuff to WhatsApp stickers.")]
pub struct Opt {
    /// IDs of emotes from 7TV to use
    #[structopt(long = "7tv")]
    pub seven_tv_ids: Vec<String>,

    /// IDs of emotes from BTTV to use
    #[structopt(long = "bttv")]
    pub bttv_ids: Vec<String>,

    /// Names of SVGs to use
    #[structopt(long = "svg")]
    pub svg_names: Vec<String>,

    /// Where to save downloaded emotes
    #[structopt(long = "dl-dir", default_value = "./dl/")]
    #[structopt(parse(try_from_str = parse_dir_path))]
    pub download_dir: PathBuf,

    /// Where to save extracted frames
    #[structopt(long = "frames-dir", default_value = "./frames/")]
    #[structopt(parse(try_from_str = parse_dir_path))]
    pub frames_dir: PathBuf,

    /// Force processing of emotes that are unlikely to fit
    #[structopt(long)]
    pub force: bool,

    /// Only parse arguments, don't process anything
    #[structopt(long)]
    pub test: bool,

    /// Only downloads the listed emotes, don't convert
    #[structopt(long)]
    pub download: bool,
}
