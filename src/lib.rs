pub mod action;
pub mod app;
pub mod event;
pub mod handler;
pub mod shell;
pub mod ui;

use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,
}

pub fn dir_pathes(dir: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<_> = dir
        .read_dir()
        .into_iter()
        .flat_map(|d| {
            d.filter_map(|p| if let Ok(p) = p { Some(p.path()) } else { None })
                .collect::<Vec<_>>()
        })
        .collect();
    files.sort();
    files.sort_by_key(|p| crate::filename(&p).starts_with("."));
    files
}

pub fn filename(path: &PathBuf) -> &str {
    path.file_name()
        .unwrap_or(OsStr::new("*unknown file*").into())
        .to_str()
        .unwrap_or("*Not compatible with UTF-8*")
}
