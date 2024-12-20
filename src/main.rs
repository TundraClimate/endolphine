use clap::Parser;
use std::path::PathBuf;

mod app;
mod error;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,
}

fn main() {
    let args = Args::parse();
    if let Err(e) = app::launch(&args.path) {
        e.handle();
    }
}
