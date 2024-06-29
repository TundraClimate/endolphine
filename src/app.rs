use crate::{action::Action, file_manager::FileManager, shell, ui::Dialog, Args};
use crossterm::{
    cursor::Hide,
    event::{Event, KeyEventKind},
    execute,
    terminal::EnterAlternateScreen,
};
use std::{error::Error, io, path::PathBuf};
use tokio::sync::mpsc::{Receiver, Sender};

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

    pub async fn run_app(self) -> Result<(), Box<dyn Error>> {
        let mut app = self;
        app.render_mode().await?;
        Ok(())
    }

    pub async fn looper(
        &mut self,
        rc: &mut Receiver<Event>,
        sender: &Sender<Option<()>>,
    ) -> Result<(), Box<dyn Error>> {
        if self.editor {
            self.open_editor().await?;
        }
        self.receive_event(rc, sender).await?;
        self.handle_action()?;
        self.auto_selector();
        self.files = FileManager::new(self);
        Ok(())
    }

    async fn receive_event(
        &mut self,
        rc: &mut Receiver<Event>,
        sender: &Sender<Option<()>>,
    ) -> Result<(), Box<dyn Error>> {
        if let Ok(event) = rc.try_recv() {
            self.handle_dialog(&event)?;
            if let Event::Key(event) = event {
                if event.kind == KeyEventKind::Press {
                    self.handle_keys(event);
                }
                if self.quit {
                    sender.send(Some(())).await?;
                } else {
                    sender.send(None).await?;
                }
            }
        }
        Ok(())
    }

    async fn open_editor(&mut self) -> io::Result<()> {
        if let Some(file) = self.cur_file() {
            shell::nvim(file).await?;
        }
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        self.editor = false;
        Ok(())
    }

    pub fn cur_file(&self) -> Option<&PathBuf> {
        self.files.require(self.cursor)
    }
}
