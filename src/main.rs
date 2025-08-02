mod args;
mod canvas;
mod clipboard;
mod component;
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

    if let Err(e) = tui::setup_logger() {
        panic!("{e}");
    }

    let args = args::parse_args();

    for arg in args.into_iter() {
        match arg {
            Ok(expected) => match expected {
                Expected::OpenEndolphine(path) => {
                    tui::enable();

                    log::info!("\n---\nLaunch endolphine: Success\n---");

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

                    log::info!("\n---\nLaunch config editor: Success\n---");

                    Command::new(editor)
                        .arg(config::file_path())
                        .status()
                        .await
                        .ok();

                    log::info!("New configuration saved");

                    let Some(config_read) = fs::read_to_string(config::file_path()).ok() else {
                        panic!("Broken configure detected: Unable to read file");
                    };

                    match config::parse_check(&config_read) {
                        Ok(_) => config::print_success_message(),
                        Err(e) => config::handle_parse_err(config_read, e),
                    }
                }
                Expected::EnableDebugMode => {
                    tui::set_dbg_hook();
                    log::info!("Debug mode enabled");
                }
                Expected::DownloadUnofficialTheme(url) => {
                    match config::download_unofficial_theme(&url).await {
                        Ok(_) => log::info!("The theme download successful"),
                        Err(e) => panic!("The theme download failed: {}", e.kind()),
                    }
                }
                Expected::DownloadOfficialTheme(name) => {
                    match config::download_official_theme(&name).await {
                        Ok(_) => log::info!("The theme download successful"),
                        Err(e) => panic!("The theme download failed: {}", e.kind()),
                    }
                }
            },
            Err(cause) => match cause {
                TerminationCause::InvalidPath(path) => {
                    panic!("Invalid path detected: {}", path.to_string_lossy())
                }
            },
        }
    }
}
