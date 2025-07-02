mod arguments;
mod config;
mod event;
mod proc;
mod state;
mod theme;
mod tui;

#[tokio::main]
async fn main() {
    use arguments::{Expected, TerminationCause};
    use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};
    use state::State;
    use std::{fs, panic, process, sync::Arc};
    use tokio::process::Command;

    panic::set_hook(Box::new(|e| {
        tui::disable();

        if let Some(e) = e.payload().downcast_ref::<String>() {
            tui::terminate(e);
        } else if let Some(e) = e.payload().downcast_ref::<&str>() {
            tui::terminate(e);
        }

        process::exit(1);
    }));

    if cfg!(windows) {
        panic!("Endolphine is not supported in Windows")
    }

    if let Err(e) = config::setup_local().await {
        panic!("Failed to create configure files: {}", e.kind());
    }

    match arguments::parse_args() {
        Expected::OpenEndolphine(path) => {
            tui::enable();

            let state = Arc::new(State::new(path));
            let handle = event::spawn_reader(state.clone());

            tui::tick_loop(state, 60, |state| {}).await;

            handle.await.ok();
        }
        Expected::OpenConfigEditor => {
            let Some(editor) = option_env!("EDITOR") else {
                panic!("$EDITOR not initialized");
            };

            Command::new(editor)
                .arg(config::file_path())
                .status()
                .await
                .ok();

            let Some(config_read) = fs::read_to_string(config::file_path()).ok() else {
                panic!("Broken configure detected: Unable to read file");
            };

            if let Err(e) = config::parse_check(&config_read) {
                let mut size_buf = 0usize;
                let span = e.span().unwrap();

                let err_lines = config_read
                    .lines()
                    .enumerate()
                    .filter_map(|(i, line)| {
                        let before_len = size_buf;
                        size_buf += line.len() + 1;

                        (0..line.len())
                            .any(|j| span.contains(&(j + before_len)))
                            .then_some(format!("{} | {}\n", i + 1, line))
                    })
                    .collect::<String>();

                eprintln!(
                    "{}{}",
                    SetForegroundColor(Color::DarkCyan),
                    SetAttribute(Attribute::Bold)
                );
                eprintln!("{:-^39}", "Invalid syntax detected");
                eprintln!("{}", e.message());
                eprintln!();
                eprintln!("{}", err_lines);
                eprintln!("{}", "-".repeat(39));
            }
        }
        Expected::Termination(cause) => match cause {
            TerminationCause::InvalidPath(path) => {
                panic!("Invalid path detected: {}", path.to_string_lossy())
            }
        },
    }
}
