use clap::Parser;
use std::path::PathBuf;

mod app;
mod canvas;
mod canvas_cache;
mod color;
mod error;
mod event_handler;
mod thread;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(e) = app::launch(&args.path).await {
        e.handle();
    }
}
