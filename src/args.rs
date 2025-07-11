use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Endolphine opens in this directory, This path must be a directory
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    path: PathBuf,

    /// Open config file with $EDITOR
    #[arg(short = 'e', long = "edit-config")]
    edit_config: bool,

    /// Enable debug mode
    #[arg(long = "dbg")]
    dbg: bool,

    /// Download an unofficial theme from URL
    #[arg(short = 'T', value_name = "URL")]
    dl_theme_from_url: Option<String>,

    /// Download an official theme.
    /// The theme list is in the README#available-themes
    #[arg(short = 't', value_name = "NAME")]
    dl_theme_official: Option<String>,
}

pub enum Expected {
    OpenEndolphine(PathBuf),
    OpenConfigEditor,
    EnableDebugMode,
}

pub enum TerminationCause {
    InvalidPath(PathBuf),
}

pub fn parse_args() -> Vec<Result<Expected, TerminationCause>> {
    let parsed = Args::parse();
    let mut res = vec![];

    if parsed.dbg {
        res.push(Ok(Expected::EnableDebugMode));
    }

    if parsed.edit_config {
        res.push(Ok(Expected::OpenConfigEditor));
    }

    let path = &parsed.path;

    match path.canonicalize() {
        Ok(path) => res.push(Ok(Expected::OpenEndolphine(path))),
        Err(_) => res.push(Err(TerminationCause::InvalidPath(path.to_path_buf()))),
    }

    res
}
