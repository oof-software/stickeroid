use anyhow::Result;
use log::LevelFilter;
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

pub fn init() -> Result<()> {
    TermLogger::init(
        LevelFilter::Trace,
        ConfigBuilder::default()
            .add_filter_allow_str("convertoid")
            .build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;
    Ok(())
}
