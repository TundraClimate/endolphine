use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(PathBuf))]
    pub path: PathBuf,

    #[arg(short = 'e')]
    pub edit_config: bool,
}

pub enum Expected {
    OpenEndolphine(PathBuf),
    OpenConfigEditor,
    Termination(TerminationCause),
}

pub enum TerminationCause {
    InvalidPath(PathBuf),
}

pub fn parse_args() -> Expected {
    let parsed = Args::parse();

    if parsed.edit_config {
        Expected::OpenConfigEditor
    } else {
        let path = &parsed.path;
        let (Ok(path), true) = (path.canonicalize(), path.is_dir()) else {
            return Expected::Termination(TerminationCause::InvalidPath(parsed.path));
        };

        Expected::OpenEndolphine(path)
    }
}
