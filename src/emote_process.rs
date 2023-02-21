use anyhow::Result;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use log::info;

pub fn test_ui_blocking() -> Result<()> {
    let selections = [
        "60898a7739b5010444d07e6e",
        "6088b8f839b5010444d078d4",
        "6027ea208fbb823604bde323",
        "59a4ea2865231102cde26e9c",
        "60b13dfcf8b3f62601c34b9f",
        "5805580c3d506fea7ee357d6",
    ];

    let selected_index = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select an emote to process")
        .default(0)
        .items(&selections)
        .interact()?;
    let selected = selections[selected_index];

    info!("Selected `{selected}`");

    Ok(())
}

pub async fn test_ui() -> Result<()> {
    tokio::task::spawn_blocking(|| test_ui_blocking())
        .await
        .unwrap()
}
