use crate::{
    actions::Action,
    event, handler,
    ui::{self, Dialog},
    Args,
};
use crossterm::event::{Event, KeyEventKind};
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
            path: args.path.canonicalize().unwrap().clone(),
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
            let (mut rc, shatdown) = event::spawn();
            ui::render_mode(|| {
                if let Ok(event) = rc.try_recv() {
                    handler::handle_dialog(&mut app, &event);
                    if let Event::Key(event) = event {
                        if event.kind == KeyEventKind::Press
                            && handler::handle_keys(&mut app, event)
                        {
                            return true;
                        }
                    }
                }
                handler::handle_action(&mut app);
                handler::auto_selector(&mut app);
                app.files = crate::dir_pathes(app.path.clone());
                false
            })?;
            shatdown.send(()).ok();
            Ok::<(), Box<dyn Error>>(())
        })?;
        Ok(())
    }
}
