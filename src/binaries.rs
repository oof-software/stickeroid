use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use derive_builder::Builder;
use futures::StreamExt;
use log::{error, info, warn};
use tokio::process::Command;

use crate::file_sequence::file_sequence;
use crate::webp;

/// Make typing key-value-pair arguments a bit nicer
trait ArgExt {
    fn arg_pair(&mut self, first: impl AsRef<OsStr>, second: impl AsRef<OsStr>) -> &mut Self;
}
impl ArgExt for Command {
    fn arg_pair(&mut self, first: impl AsRef<OsStr>, second: impl AsRef<OsStr>) -> &mut Self {
        self.arg(first).arg(second)
    }
}

/// Call the binary with the `-version` argument
async fn check_version(binary: &Path) -> Result<String> {
    let output = Command::new(binary).arg("-version").output().await;
    let name = binary.file_name().unwrap().to_str().unwrap();

    match &output {
        Ok(_) => info!("found binary `{name}`"),
        Err(err) => error!("couldn't find binary `{binary:?}`: {err}"),
    }

    let stdout = String::from_utf8(output?.stdout)?;
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
    pub async fn info(&self, webp: impl AsRef<Path>) -> Result<webp::WebpInfo> {
        let output = Command::new(&self.0).arg(webp.as_ref()).output().await?;
        let stdout = String::from_utf8(output.stdout)?;
        webp::WebpInfo::from_stdout(&stdout, webp.as_ref())
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
    pub async fn dump_frames(&self, webp: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
        Command::new(&self.0)
            .arg_pair("-prefix", "")
            .arg_pair("-folder", dst.as_ref())
            .arg(webp.as_ref())
            .output()
            .await?;

        Ok(())
    }
}

#[derive(Debug, Builder)]
pub struct FfmpegOptions {
    #[builder(default = "(512, 512)")]
    pub scale: (i32, i32),
    #[builder(default = "75")]
    pub quality: i32,
    #[builder(default = "4")]
    pub compression_level: i32,
    #[builder(default = "-1")]
    pub preset: i32,
    #[builder(default = "30")]
    pub delay_ms: i32,
    #[builder(default = "50")]
    pub fps: i32,
    #[builder(default = "0")]
    pub lossless: i32,
    #[builder(default = "0")]
    pub loop_count: i32,
}

impl FfmpegOptions {
    pub fn builder() -> FfmpegOptionsBuilder {
        FfmpegOptionsBuilder::default()
    }
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
    pub async fn resize_images(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        const VIDEO_FILTER: &str = "\
            pad=512:512:-1:-1:color=0x00000000,\
            scale=w=512:h=512:force_original_aspect_ratio=decrease";

        Command::new(&self.0)
            .arg_pair("-i", input.as_ref())
            .arg_pair("-vf", VIDEO_FILTER)
            .arg("-y")
            .arg(output.as_ref())
            .output()
            .await?;

        Ok(())
    }
    pub async fn webp_from_images(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        opt: FfmpegOptions,
    ) -> Result<()> {
        if opt.fps > (1000 / opt.delay_ms) {
            warn!("fps too high for given delay");
        }

        Command::new(&self.0)
            .arg_pair("-i", input.as_ref())
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
            .arg(output.as_ref())
            .output()
            .await?;

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
    /// # Arguments
    /// - `q` Compression factor
    /// - `m` Compression method
    pub async fn webp_from_images<I, P, Q>(&self, input: I, output: P, q: u32, m: u32) -> Result<()>
    where
        I: IntoIterator<Item = (Q, u32)>,
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-o", output.as_ref())
            .arg_pair("-loop", "0")
            .arg("-mixed");
        let q_str = format!("{q}");
        let m_str = format!("{m}");
        for (frame_path, duration) in input {
            cmd.arg_pair("-d", format!("{duration}"))
                .arg_pair("-q", &q_str)
                .arg_pair("-m", &m_str)
                .arg(frame_path.as_ref());
        }
        cmd.output().await?;

        Ok(())
    }
    pub async fn webp_from_dir(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        q: u32,
        m: u32,
        d: u32,
    ) -> Result<()> {
        let sequence = file_sequence(input.as_ref()).await?;
        self.webp_from_images(sequence.into_iter().map(|seq| (seq.path, d)), output, q, m)
            .await
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
    pub async fn render_svg(
        &self,
        input: impl AsRef<OsStr>,
        output: impl AsRef<OsStr>,
    ) -> Result<()> {
        Command::new(&self.0)
            .arg_pair("-size", "512x512")
            .arg_pair("-background", "none")
            .arg(input.as_ref())
            .arg_pair("-gravity", "center")
            .arg_pair("-extent", "512x512")
            .arg_pair("-define", "webp:lossless=true")
            .arg(output.as_ref())
            .output()
            .await?;

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
    pub async fn view_webp(&self, input: impl AsRef<OsStr>) -> Result<()> {
        Ok(Command::new(&self.0)
            .arg(input.as_ref())
            .output()
            .await
            .map(|_| ())?)
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

    pub async fn check(&self, parallel: usize) -> Result<HashMap<&'static str, String>> {
        async fn inner(name: &'static str, path: &Path) -> Result<(&'static str, String)> {
            Ok((name, check_version(path).await?))
        }

        let to_check = [
            inner("anim_dump", self.anim_dump.path()),
            inner("webp_info", self.webp_info.path()),
            inner("ffmpeg", self.ffmpeg.path()),
            inner("magick", self.magick.path()),
            inner("img_2_webp", self.img_2_webp.path()),
            inner("v_webp", self.v_webp.path()),
        ];

        let mut map = HashMap::with_capacity(to_check.len());
        let results = futures::stream::iter(to_check)
            .buffer_unordered(parallel)
            .collect::<Vec<_>>()
            .await;

        for result in results {
            match result {
                Ok((name, val)) => map.insert(name, val),
                Err(err) => return Err(err),
            };
        }

        Ok(map)
    }
}
