use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    path: PathBuf,

    #[arg(short = 'e')]
    edit_config: bool,

    #[arg(long = "dbg")]
    dbg: bool,

    #[arg(short = 'T', long)]
    theme: Option<String>,
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
