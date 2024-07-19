pub mod action;
pub mod app;
pub mod command;
pub mod file_manager;
pub mod finder;
pub mod handler;
pub mod ui;

use crate::{action::menu, app::App};
use clap::Parser;
use std::{ffi::OsStr, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,
}

pub fn dir_pathes(app: Option<&App>, dir: &PathBuf) -> Vec<PathBuf> {
    if app.is_some() && app.unwrap().menu_opened() {
        if let Some(ref path) = app.unwrap().menu {
            return menu::choices(path).unwrap_or(vec![]);
        }
    }
    let mut files: Vec<_> = dir
        .read_dir()
        .into_iter()
        .flat_map(|d| d.filter_map(Result::ok).map(|e| e.path()))
        .filter(|p| match app {
            Some(app) if app.is_search() => match &app.finder {
                Some(finder) => {
                    let regex = finder.regex();
                    regex.map_or(true, |r| r.is_match(filename(p)))
                }
                None => true,
            },
            _ => true,
        })
        .collect();
    files.sort();
    sort_by_case(&mut files);
    files.sort_by_key(|p| crate::filename(&p).starts_with("."));
    files
}

fn sort_by_case(files: &mut Vec<PathBuf>) {
    files.sort_by_key(|p| {
        crate::filename(&p)
            .chars()
            .next()
            .unwrap_or(' ')
            .is_uppercase()
    });
}

pub fn filename(path: &PathBuf) -> &str {
    path.file_name()
        .unwrap_or(OsStr::new("*unknown file*").into())
        .to_str()
        .unwrap_or("*Not compatible with UTF-8*")
}
