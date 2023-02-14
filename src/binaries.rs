use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use anyhow::Result;
use derive_builder::Builder;
use log::warn;

use crate::file_sequence::file_sequence;
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

/// Call the binary with the `-version` argument
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
        Command::new(&self.0)
            .arg_pair("-prefix", "")
            .arg_pair("-folder", dst)
            .arg(webp)
            .output()?;

        Ok(())
    }
}

#[derive(Debug, Builder)]
pub struct FfmpegOptions {
    #[builder(default = "(512, 512)")]
    scale: (u32, u32),
    #[builder(default = "75")]
    quality: u32,
    #[builder(default = "4")]
    compression_level: u32,
    #[builder(default = "-1")]
    preset: i32,
    #[builder(default = "30")]
    delay_ms: u32,
    #[builder(default = "50")]
    fps: u32,
    #[builder(default = "0")]
    lossless: u32,
    #[builder(default = "0")]
    loop_count: u32,
}

impl FfmpegOptions {
    pub fn video_filter(&self) -> String {
        format!(
            "fps={},\
            setpts=PTS*({}/40),\
            scale=w={}:h={}:force_original_aspect_ratio=decrease,\
            pad=512:512:-1:-1:color=0x00000000",
            self.fps, self.delay_ms, self.scale.0, self.scale.1
        )
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

        Command::new(&self.0)
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
        opt: FfmpegOptions,
    ) -> Result<()> {
        if opt.fps > (1000 / opt.delay_ms) {
            warn!("fps too high for given delay");
        }

        Command::new(&self.0)
            .arg_pair("-i", input)
            .arg_pair("-pix_fmt", "yuva420p")
            .arg_pair("-compression_level", format!("{}", opt.compression_level))
            .arg_pair("-preset", format!("{}", opt.preset))
            .arg_pair("-quality", format!("{}", opt.quality))
            .arg_pair("-loop", format!("{}", opt.loop_count))
            .arg_pair("-lossless", format!("{}", opt.lossless))
            .arg_pair("-vf", opt.video_filter())
            .arg_pair("-fps_mode", "vfr")
            .arg("-an")
            .arg("-y")
            .arg(output)
            .output()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Img2Webp(PathBuf);

impl Img2Webp {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn webp_from_images<I, S, T>(&self, input: I, output: S, q: u32, m: u32) -> Result<()>
    where
        I: IntoIterator<Item = (T, u32)>,
        S: AsRef<OsStr>,
        T: AsRef<OsStr>,
    {
        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-o", output)
            .arg_pair("-loop", "0")
            .arg("-mixed");
        let q_str = format!("{q}");
        let m_str = format!("{m}");
        for (frame_path, duration) in input {
            cmd.arg_pair("-d", format!("{duration}"))
                .arg_pair("-q", &q_str)
                .arg_pair("-m", &m_str)
                .arg(frame_path);
        }
        cmd.output()?;

        Ok(())
    }
    pub fn webp_from_dir<S, T>(&self, input: S, output: T, q: u32, m: u32, d: u32) -> Result<()>
    where
        S: AsRef<OsStr>,
        T: AsRef<OsStr>,
    {
        let sequence = file_sequence(input)
            .into_iter()
            .map(|(_, entry)| (entry.into_path(), d))
            .collect::<Vec<_>>();
        self.webp_from_images(sequence.into_iter(), output, q, m)
    }
}

#[derive(Debug)]
pub struct Magick(PathBuf);

impl Magick {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn render_svg<S: AsRef<OsStr>, T: AsRef<OsStr>>(&self, input: S, output: T) -> Result<()> {
        Command::new(&self.0)
            .arg_pair("-size", "512x512")
            .arg_pair("-background", "none")
            .arg(input)
            .arg_pair("-gravity", "center")
            .arg_pair("-extent", "512x512")
            .arg_pair("-define", "webp:lossless=true")
            .arg(output)
            .output()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct VWebp(PathBuf);

impl VWebp {
    pub fn new(str: &str) -> Self {
        Self(PathBuf::from_str(str).unwrap())
    }
    pub fn path(&self) -> &Path {
        &self.0
    }
    pub fn view_webp<S: AsRef<OsStr>>(&self, input: S) -> Result<()> {
        Command::new(&self.0).arg(input).output()?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Binaries {
    pub anim_dump: AnimDump,
    pub webp_info: WebpInfo,
    pub ffmpeg: Ffmpeg,
    pub magick: Magick,
    pub img_2_webp: Img2Webp,
    pub v_webp: VWebp,
}

impl Binaries {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            anim_dump: AnimDump::new(&dotenv::var("ANIM_DUMP_BIN")?),
            webp_info: WebpInfo::new(&dotenv::var("WEBP_INFO_BIN")?),
            ffmpeg: Ffmpeg::new(&dotenv::var("FFMPEG_BIN")?),
            magick: Magick::new(&dotenv::var("MAGICK_BIN")?),
            img_2_webp: Img2Webp::new(&dotenv::var("IMG2WEBP_BIN")?),
            v_webp: VWebp::new(&dotenv::var("VWEBP_BIN")?),
        })
    }

    pub fn check(&self) -> Result<HashMap<&str, String>> {
        Ok(HashMap::from_iter([
            ("anim_dump", check_version(self.anim_dump.path())?),
            ("webp_info", check_version(self.webp_info.path())?),
            ("ffmpeg", check_version(self.ffmpeg.path())?),
            ("magick", check_version(self.magick.path())?),
            ("img_2_webp", check_version(self.img_2_webp.path())?),
            ("v_webp", check_version(self.v_webp.path())?),
        ]))
    }
}
