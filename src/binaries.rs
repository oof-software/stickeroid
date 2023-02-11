use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use anyhow::Result;

use crate::webp_frames::WebpFrames;

/// Make typing key-value-pair arguments a bit nicer
trait ArgExt {
    fn arg_pair<S: AsRef<OsStr>, T: AsRef<OsStr>>(&mut self, first: S, second: T) -> &mut Self;
}
impl ArgExt for Command {
    fn arg_pair<S: AsRef<OsStr>, T: AsRef<OsStr>>(&mut self, first: S, second: T) -> &mut Self {
        self.arg(first).arg(second)
    }
}

fn check_version(binary: &Path) -> Result<String> {
    let output = Command::new(binary).arg("-version").output()?;
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

#[derive(Debug)]
pub struct WebpInfo(PathBuf);

impl WebpInfo {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn run<S: AsRef<OsStr>>(&self, webp: S) -> Result<WebpFrames> {
        let output = Command::new(&self.0).arg(webp).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        WebpFrames::from_webp_info(&stdout)
    }
}

#[derive(Debug)]
pub struct AnimDump(PathBuf);

impl AnimDump {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn run<S: AsRef<OsStr>, T: AsRef<OsStr>>(&self, webp: S, dst: T) -> Result<()> {
        let _ = Command::new(&self.0)
            .arg_pair("-prefix", "")
            .arg_pair("-folder", dst)
            .arg(webp)
            .output()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Ffmpeg(PathBuf);

impl Ffmpeg {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn resize_images<S: AsRef<OsStr>, T: AsRef<OsStr>>(
        &self,
        input: S,
        output: T,
    ) -> Result<()> {
        const VIDEO_FILTER: &str = "\
            pad=512:512:-1:-1:color=0x00000000,\
            scale=w=512:h=512:force_original_aspect_ratio=decrease";

        let _ = Command::new(&self.0)
            .arg_pair("-i", input)
            .arg_pair("-vf", VIDEO_FILTER)
            .arg("-y")
            .arg(output)
            .output()?;
        Ok(())
    }
    pub fn webp_from_images<S: AsRef<OsStr>, T: AsRef<OsStr>>(
        &self,
        input: S,
        output: T,
        quality: u8,
        delay: u32,
    ) -> Result<()> {
        let video_filter = format!("fps=(1/{delay})*1000,setpts=PTS*({delay}/40)");

        let _ = Command::new(&self.0)
            .arg_pair("-i", input)
            .arg_pair("-pix_fmt", "yuva420p")
            .arg_pair("-compression_level", "4")
            .arg_pair("-loop", "0")
            .arg("-an")
            .arg_pair("-preset", "-1")
            .arg_pair("-q:v", format!("{quality}"))
            .arg_pair("-vf", video_filter)
            .arg_pair("-fps_mode", "vfr")
            .arg("-y")
            .arg(output)
            .output()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Binaries {
    pub anim_dump: AnimDump,
    pub webp_info: WebpInfo,
    pub ffmpeg: Ffmpeg,
    pub magick: PathBuf,
    pub img_2_webp: PathBuf,
    pub v_webp: PathBuf,
}

impl Binaries {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            anim_dump: AnimDump::new(&dotenv::var("ANIM_DUMP_BIN")?),
            webp_info: WebpInfo::new(&dotenv::var("WEBP_INFO_BIN")?),
            ffmpeg: Ffmpeg::new(&dotenv::var("FFMPEG_BIN")?),
            magick: PathBuf::from_str(&dotenv::var("MAGICK_BIN")?)?,
            img_2_webp: PathBuf::from_str(&dotenv::var("IMG2WEBP_BIN")?)?,
            v_webp: PathBuf::from_str(&dotenv::var("VWEBP_BIN")?)?,
        })
    }

    pub fn check(&self) -> Result<HashMap<&str, String>> {
        Ok(HashMap::from_iter([
            ("anim_dump", check_version(self.anim_dump.path())?),
            ("webp_info", check_version(self.webp_info.path())?),
            ("ffmpeg", check_version(self.ffmpeg.path())?),
            ("magick", check_version(&self.magick)?),
            ("img_2_webp", check_version(&self.img_2_webp)?),
            ("v_webp", check_version(&self.v_webp)?),
        ]))
    }
}
