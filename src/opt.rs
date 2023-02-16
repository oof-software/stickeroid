use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "convertoid", about = "Convert stuff to WhatsApp stickers.")]
pub struct Opt {
    /// IDs of emotes from 7TV to use
    #[structopt(long = "7tv")]
    pub seven_tv_ids: Vec<String>,

    #[structopt(long = "bttv")]
    pub bttv_ids: Vec<String>,

    /// Names of SVG files to use
    #[structopt(long = "svg")]
    pub svg_names: Vec<String>,

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
