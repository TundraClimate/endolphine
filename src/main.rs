mod args;
mod canvas;
mod config;
mod event;
mod misc;
mod proc;
mod state;
mod tui;

#[tokio::main]
async fn main() {
    use args::{Expected, TerminationCause};
    use state::State;
    use std::{fs, sync::Arc};
    use tokio::process::Command;

    tui::set_panic_hook();

    if cfg!(windows) {
        panic!("Endolphine is not supported in Windows")
    }

    if let Err(e) = config::setup_local().await {
        panic!("Failed to create configure files: {}", e.kind());
    }

    match args::parse_args() {
        Expected::OpenEndolphine(path) => {
            tui::enable();

            let state = Arc::new(State::new(path));
            let handle = event::spawn_reader(state.clone());

            tui::tick_loop(state, 60, |state| {
                canvas::draw(state);
            })
            .await;

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
                config::handle_parse_err(config_read, e);
            }
        }
        Expected::Termination(cause) => match cause {
            TerminationCause::InvalidPath(path) => {
                panic!("Invalid path detected: {}", path.to_string_lossy())
            }
        },
    }
}
