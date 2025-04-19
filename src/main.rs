mod app;
mod builtin;
mod canvas;
mod clipboard;
mod command;
mod config;
mod cursor;
mod input;
mod key;
mod menu;
mod misc;
mod theme;

use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".", value_parser = clap::value_parser!(std::path::PathBuf))]
    pub path: std::path::PathBuf,

    #[arg(short = 'e')]
    pub edit_config: bool,
}

#[tokio::main]
async fn main() {
    start().await.unwrap_or_else(|e| e.handle());
}

fn terminate<D: std::fmt::Display>(e: D) {
    eprintln!(
        "{}{}",
        crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
        crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
    );
    eprintln!("{:-^41}", "Endolphine terminated");
    eprintln!(" {}", e);
    eprintln!("{}", "-".repeat(41));
    crate::sys_log!("e", "Endolphine terminated\n{}", e);
}

fn check_config() -> ! {
    if let Err((e, lines)) = config::check() {
        eprintln!(
            "{}{}",
            crossterm::style::SetForegroundColor(crossterm::style::Color::DarkCyan),
            crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
        );
        eprintln!("{:-^39}", "Invalid syntax detected");
        eprintln!("{}", e.message());
        eprintln!();
        eprintln!("{}", lines);
        eprintln!();
        eprintln!("{}", "-".repeat(39));

        std::process::exit(1)
    } else {
        std::process::exit(0)
    }
}

async fn start() -> Result<(), Error> {
    if cfg!(windows) {
        panic!("Endolphine is not supported in Windows")
    }

    std::panic::set_hook(Box::new(|e| {
        app::disable_tui().ok();

        if let Some(e) = e.payload().downcast_ref::<String>() {
            terminate(e);
        } else if let Some(e) = e.payload().downcast_ref::<&str>() {
            terminate(e);
        }
        std::process::exit(1);
    }));

    app::config_init()?;

    let args = Args::parse();

    if args.edit_config {
        config::edit().await;
        check_config();
    }

    app::launch(&args.path).await?;

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to change the screen mode")]
    ScreenModeChangeFailed,

    #[error("filesystem error: {0}")]
    #[allow(clippy::enum_variant_names)]
    FilesystemError(String),

    #[error("The struct parsing failed: {0}")]
    TomlParseFailed(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Found error in running \"{0}\": {1}")]
    CommandExecutionFailed(String, String),

    #[error("Display the log failed")]
    LogDisplayFailed,

    #[error("The row rendering failed")]
    RowRenderingFailed,

    #[error("The input-area rendering failed")]
    InputRenderingFailed,

    #[error("Found platform error: {0}")]
    #[allow(clippy::enum_variant_names)]
    PlatformError(String),

    #[error("Screen flush failed: {0}")]
    ScreenFlushFailed(String),

    #[error("out log failed: {0}")]
    OutLogToFileFailed(String),

    #[error("found incorrect program code: {0}:{1}")]
    IncorrectProgram(String, String),
}

impl Error {
    pub fn handle(self) {
        match self {
            Self::CommandExecutionFailed(cmd, kind) => {
                crate::sys_log!("w", "Can't be execute \"{}\": {}", cmd, kind);
                crate::log!("Failed to run \"{}\": {}", cmd, kind);
            }
            Self::ScreenModeChangeFailed => {
                crate::sys_log!(
                    "e",
                    "Couldn't change the screen mode: disabled the mode in terminal or operation system"
                );

                panic!("{}", self);
            }
            Self::LogDisplayFailed | Self::RowRenderingFailed | Self::InputRenderingFailed => {
                crate::sys_log!("e", "Rendering failed");

                panic!("{}", self);
            }
            Self::ScreenFlushFailed(_) => {
                crate::sys_log!("e", "The stdout can't flush");

                panic!("{}", self);
            }
            Self::IncorrectProgram(loc, info) => {
                crate::sys_log!("e", "Found incorrect program");

                app::disable_tui().ok();

                eprintln!(
                    "{}{}",
                    crossterm::style::SetForegroundColor(crossterm::style::Color::Red),
                    crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
                );
                eprintln!("{:-^41}", "FOUND INCORRECT PROGRAM");
                let issue_url = "https://github.com/TundraClimate/endolphine/issues";
                eprintln!(" Please report here: {}", issue_url);
                eprintln!(" The error was found: {}", loc);
                eprintln!(" Error infomation: {}", info);
                eprintln!("{}", "-".repeat(41));

                std::process::exit(1)
            }
            _ => panic!("{}", self),
        }
    }
}
