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
    pub selected: Vec<usize>,
    pub is_cut: bool,
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
            selected: vec![],
            is_cut: false,
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
                handler::handle_action(&mut app);
                handler::auto_selector(&mut app);
                false
            })?;
            shatdown.send(()).ok();
            Ok::<(), Box<dyn Error>>(())
        })?;
        Ok(())
    }
}
