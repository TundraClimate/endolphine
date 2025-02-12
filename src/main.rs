use clap::Parser;
use std::path::PathBuf;

mod app;
mod canvas;
mod clipboard;
mod color;
mod config;
mod cursor;
mod error;
mod global;
mod handler;
mod input;
mod menu;
mod misc;

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
    app::config_init().unwrap_or_else(|e| e.handle());

    let args = Args::parse();

    if args.edit_config {
        let editor = option_env!("EDITOR").unwrap_or("vi");

        let Some(config_path) = config::file_path() else {
            panic!("Open error: Config not initialized");
        };

        tokio::process::Command::new(editor)
            .arg(config_path)
            .status()
            .await
            .ok();

        if let Some(Err(e)) = config::Config::try_load() {
            let config = config::file_path().and_then(|p| std::fs::read_to_string(p).ok());
            let position_lines = if let (Some(config), Some(span)) = (config, e.span()) {
                let lines = config
                    .char_indices()
                    .collect::<Vec<_>>()
                    .split(|(_, c)| *c == '\n')
                    .filter_map(|line| {
                        line.iter()
                            .any(|(i, _)| span.contains(i))
                            .then_some(line.iter().map(|(_, c)| *c).collect::<String>())
                    })
                    .collect::<Vec<_>>();
                lines.join("\n")
            } else {
                String::new()
            };
            println!("{}\n---\n{}\n---", e.message(), position_lines);
        }

        return;
    }

    if let Err(e) = app::launch(&args.path).await {
        e.handle();
    }
}
