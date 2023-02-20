use std::path::PathBuf;

use structopt::StructOpt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DirPathParseError {
    #[error("couldn't create directory: {0}")]
    Create(std::io::Error),
    #[error("exists, but is not a directory")]
    InvalidType,
}

#[derive(Error, Debug)]
pub enum FilePathParseError {
    #[error("couldn't fetch metadata: {0}")]
    NoMetadata(#[from] std::io::Error),
    #[error("doesn't correspond to a file")]
    InvalidType,
}

fn valid_dir_path<P>(src: P) -> Result<PathBuf, DirPathParseError>
where
    P: AsRef<str>,
{
    let path = PathBuf::from(src.as_ref());
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
                return Err(DirPathParseError::Create(err));
            } else {
                Ok(path)
            }
        }
    }
}

fn valid_file_path<P>(src: P) -> Result<PathBuf, FilePathParseError>
where
    P: AsRef<str>,
{
    let path = PathBuf::from(src.as_ref());
    let meta = path.metadata()?;
    if !meta.is_file() {
        Err(FilePathParseError::InvalidType)
    } else {
        Ok(path)
    }
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
    #[structopt(long = "dl-dir", parse(try_from_str = valid_dir_path))]
    pub download_dir: PathBuf,

    /// Where to save extracted frames
    #[structopt(long = "frames-dir", parse(try_from_str = valid_dir_path))]
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
