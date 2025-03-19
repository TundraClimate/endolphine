use clap::Parser;
use std::path::PathBuf;

mod app;
mod builtin;
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
    if cfg!(windows) {
        panic!("Endolphine is not supported in Windows")
    }

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
}

fn out_checked() -> ! {
    if let Err((e, lines)) = config::check() {
        eprintln!(
            "{}{}",
            crossterm::style::SetForegroundColor(crossterm::style::Color::DarkCyan),
            crossterm::style::SetAttribute(crossterm::style::Attribute::Bold),
        );
        eprintln!("-------Invalid syntax detected-------");
        eprintln!("{}", e.message());
        eprintln!();
        eprintln!("{}", lines);
        eprintln!();
        eprintln!("-------------------------------------");

        std::process::exit(1)
    } else {
        std::process::exit(0)
    }
}

async fn start() -> Result<(), app::Error> {
    std::panic::set_hook(Box::new(|e| {
        crate::disable_tui!().ok();

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
        out_checked();
    }

    app::launch(&args.path).await?;

    Ok::<(), app::Error>(())
}
