use crate::{
    action::Action,
    event::{self, Signal},
    file_manager::FileManager,
    ui::Dialog,
    Args,
};
use crossterm::event::{Event, KeyEventKind};
use std::{error::Error, path::PathBuf};
use tokio::runtime::Runtime;

pub struct App {
    pub path: PathBuf,
    pub files: FileManager,
    pub cursor: usize,
    pub action: Action,
    pub dialog: Option<Dialog>,
    pub register: Vec<PathBuf>,
    pub selected: Vec<usize>,
    pub is_cut: bool,
    pub is_search: bool,
    pub editor: bool,
    pub quit: bool,
}

impl App {
    pub fn new(args: Args) -> App {
        App {
            path: args.path.canonicalize().unwrap().clone(),
            files: FileManager::from(&args.path),
            cursor: 0,
            action: Action::None,
            dialog: None,
            register: vec![],
            selected: vec![],
            is_cut: false,
            is_search: false,
            editor: false,
            quit: false,
        }
    }

    pub fn init(self) -> Result<App, Box<dyn Error>> {
        Ok(self)
    }

    pub fn run_app(self) -> Result<(), Box<dyn Error>> {
        let mut app = self;
        Runtime::new()?.block_on(async {
            let (mut rc, sender) = event::spawn();
            let looper = |app: &mut App| {
                if let Ok(event) = rc.try_recv() {
                    app.handle_dialog(&event)?;
                    if let Event::Key(event) = event {
                        if event.kind == KeyEventKind::Press {
                            app.handle_keys(event);
                        }
                    }
                }
                app.handle_action()?;
                app.auto_selector();
                app.files = FileManager::new(app);
                Ok(())
            };
            app.render_mode(looper, &sender).await?;
            sender.send(Signal::Shatdown).await?;
            Ok::<(), Box<dyn Error>>(())
        })?;
        Ok(())
    }

    pub fn cur_file(&self) -> Option<&PathBuf> {
        self.files.cur_file(self.cursor)
    }
}
