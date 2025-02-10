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
    app::config_init().unwrap_or_else(|e| e.handle());

    let args = Args::parse();

    if args.edit_config {
        let editor = option_env!("EDITOR").unwrap_or("vi");

        let Some(config_path) = config::file_path() else {
            panic!("Open error: Config not initialized");
        };

        tokio::process::Command::new(editor)
            .arg(config_path)
            .status()
            .await
            .ok();

        return;
    }

    if let Err(e) = app::launch(&args.path).await {
        e.handle();
    }
}
