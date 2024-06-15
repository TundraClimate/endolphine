use crate::{actions::Action, handler, ui, Args};
use crossterm::event::KeyEventKind;
use std::{error::Error, path::PathBuf};
use tokio::runtime::Runtime;

pub struct App {
    opened_path: PathBuf,
    current_path: PathBuf,
    pub action: Action,
}

impl App {
    pub fn new(args: Args) -> App {
        App {
            opened_path: args.path.clone(),
            current_path: args.path.clone(),
            action: Action::None,
        }
    }

    pub fn init(self) -> Result<App, Box<dyn Error>> {
        Ok(self)
    }

    pub fn run_app(self) -> Result<(), Box<dyn Error>> {
        let mut app = self;
        Runtime::new()?.block_on(async {
            let (mut rc, shatdown) = crossterm_keyreader::spawn();
            ui::render_mode(|| {
                if let Ok(event) = rc.try_recv() {
                    if event.kind == KeyEventKind::Press && handler::handle_keys(&mut app, event) {
                        return true;
                    }
                }
                false
            })?;
            shatdown.send(()).ok();
            Ok::<(), Box<dyn Error>>(())
        })?;
        Ok(())
    }
}
