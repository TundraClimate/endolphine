use crate::{
    actions::Action,
    handler,
    ui::{self, Dialog},
    Args,
};
use crossterm::event::KeyEventKind;
use std::{error::Error, path::PathBuf};
use tokio::runtime::Runtime;

pub struct App {
    pub path: PathBuf,
    pub files: Vec<PathBuf>,
    pub cursor: usize,
    pub action: Action,
    pub dialog: Option<Dialog>,
    pub register: Vec<PathBuf>,
}

impl App {
    pub fn new(args: Args) -> App {
        App {
            path: args.path.clone(),
            files: crate::dir_pathes(args.path),
            cursor: 0,
            action: Action::None,
            dialog: None,
            register: vec![],
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
