use clap::Parser;
use std::path::PathBuf;

mod app;
mod canvas;
mod clipboard;
mod config;
mod cursor;
mod error;
mod handler;
mod input;
mod menu;
mod misc;
mod theme;

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
    std::panic::set_hook(Box::new(|e| {
        crate::disable_tui!().ok();

        if let Some(e) = e.payload().downcast_ref::<String>() {
            eprintln!("Endolphine terminated:");
            eprintln!("- {}", e);
        }
        std::process::exit(1);
    }));

    app::config_init()?;

    let args = Args::parse();

    if args.edit_config {
        config::edit().await?;
        if let Err((e, lines)) = config::check() {
            println!("{}\n---\n{}\n---", e.message(), lines);
        }
        return Ok(());
    }

    if !misc::exists_item(&args.path) || args.path.is_file() {
        return Err(error::EpError::Init(format!(
            "invalid path (-> {})",
            args.path.to_string_lossy()
        )));
    }

    app::launch(&args.path).await?;

    Ok(())
}
