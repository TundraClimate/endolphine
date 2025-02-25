use clap::Parser;
use std::path::PathBuf;

mod app;
mod canvas;
mod clipboard;
mod color;
mod config;
mod cursor;
mod error;
mod global;
mod handler;
mod input;
mod menu;
mod misc;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,

    #[arg(short = 'e')]
    pub edit_config: bool,
}

#[tokio::main]
async fn main() {
    start().await.unwrap_or_else(|e| e.handle());
}

async fn start() -> error::EpResult<()> {
    app::config_init()?;

    let args = Args::parse();

    if args.edit_config {
        config::edit_and_check().await?;
        return Ok(());
    }

    app::launch(&args.path).await?;

    Ok(())
}
