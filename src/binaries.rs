use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use futures::StreamExt;
use log::{error, info, warn};
use simple_error::simple_error;
use tokio::process::Command;

use crate::convert::ConversionOptions;
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

async fn run_command(mut cmd: Command) -> Result<()> {
    let program = cmd.as_std().get_program().to_str().unwrap().to_string();
    let args = cmd
        .as_std()
        .get_args()
        .fold(program.clone(), |mut acc, next| {
            acc.push(' ');
            acc.push_str(next.to_str().unwrap());
            acc
        });

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        error!("process `{program}` fucked up");
        error!("args: `{args}`");
        error!("stderr: `{stderr}`");
        Err(simple_error!("process `{program}` fucked up").into())
    } else {
        Ok(())
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
        webp::WebpInfo::from_stdout(&stdout)
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
        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-prefix", "")
            .arg_pair("-folder", dst.as_ref())
            .arg(webp.as_ref());
        run_command(cmd).await
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
            scale=w=512:h=512:force_original_aspect_ratio=decrease,\
            pad=512:512:-1:-1:color=0x00000000";

        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-i", input.as_ref())
            .arg_pair("-vf", VIDEO_FILTER)
            .arg("-y")
            .arg(output.as_ref());
        run_command(cmd).await
    }
    pub async fn webp_from_images(
        &self,
        opt: &ConversionOptions,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        if opt.fps > (1000 / opt.delay_ms) {
            warn!("fps too high for given delay");
        }

        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-i", input.as_ref())
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
            .arg(output.as_ref());
        run_command(cmd).await
    }
}

pub struct Img2WebpFrame {
    pub path: PathBuf,
    pub duration: i32,
    pub compression_quality: i32,
    pub compression_method: i32,
}
impl Img2WebpFrame {
    pub fn new(
        path: impl AsRef<Path>,
        duration: i32,
        compression_quality: i32,
        compression_method: i32,
    ) -> Img2WebpFrame {
        Img2WebpFrame {
            path: path.as_ref().to_path_buf(),
            duration,
            compression_quality,
            compression_method,
        }
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

    pub async fn webp_from_images(
        &self,
        opt: &ConversionOptions,
        output: impl AsRef<Path>,
        frames: &[Img2WebpFrame],
    ) -> Result<()> {
        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-o", output.as_ref())
            .arg_pair("-loop", opt.loop_count.to_string());

        for frame_opt in frames {
            cmd.arg_pair("-d", frame_opt.duration.to_string())
                .arg_pair("-q", frame_opt.compression_quality.to_string())
                .arg_pair("-m", frame_opt.compression_method.to_string())
                .arg(&frame_opt.path);
        }

        run_command(cmd).await
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
    pub async fn convert(
        &self,
        input: impl AsRef<Path>,
        output: impl AsRef<Path>,
        lossless: bool,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.0);
        cmd.arg_pair("-size", "512x512")
            .arg_pair("-background", "none")
            .arg(input.as_ref())
            .arg_pair("-gravity", "center")
            .arg_pair("-extent", "512x512");
        if lossless {
            cmd.arg_pair("-define", "webp:lossless=true");
        }
        cmd.arg(output.as_ref());
        run_command(cmd).await
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
        let mut cmd = Command::new(&self.0);
        cmd.arg(input.as_ref());
        run_command(cmd).await
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
