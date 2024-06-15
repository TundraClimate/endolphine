pub mod actions;
pub mod app;
pub mod handler;
pub mod ui;

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,
}

pub fn dir_pathes(dir: PathBuf) -> Vec<PathBuf> {
    let mut vec = vec![];
    for entry in dir.read_dir().unwrap() {
        if let Ok(entry) = entry {
            vec.push(entry.path());
        }
    }
    vec
}
