use anyhow::Result;
use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

pub fn init() -> Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    Ok(())
}
