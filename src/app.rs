use crate::{action::Action, file_manager::FileManager, finder::Finder, shell, ui::Dialog, Args};
use crossterm::{
    cursor::Hide,
    event::{Event, KeyEventKind},
    execute,
    terminal::EnterAlternateScreen,
};
use std::{error::Error, io, path::PathBuf};
use termkit::EventThread;
use tokio::time::{self, Duration, Instant};

pub struct App {
    pub path: PathBuf,
    pub files: FileManager,
    pub cursor: usize,
    pub action: Action,
    pub dialog: Option<Dialog>,
    pub selected: Vec<usize>,
    pub menu: Option<PathBuf>,
    pub finder: Option<Finder>,
    pub is_cut: bool,
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
            selected: vec![],
            menu: None,
            finder: None,
            is_cut: false,
            editor: false,
            quit: false,
        }
    }

    pub fn init(self) -> Result<App, Box<dyn Error>> {
        Ok(self)
    }

    pub async fn run_app(self) -> Result<(), Box<dyn Error>> {
        let mut app = self;
        let mut ev = EventThread::spawn();
        app.render_mode(&mut ev).await?;
        Ok(())
    }

    pub async fn render_mode(&mut self, ev: &mut EventThread) -> Result<(), Box<dyn Error>> {
        termkit::enable_tui()?;

        while !self.quit {
            let start = Instant::now();
            self.ui()?;
            self.looper(ev).await?;
            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(10) {
                time::sleep(Duration::from_millis(10) - elapsed).await;
            }
        }

        termkit::disable_tui()?;
        Ok(())
    }

    pub async fn looper(&mut self, ev: &mut EventThread) -> Result<(), Box<dyn Error>> {
        if self.editor {
            self.open_editor().await?;
        }
        self.receive_event(ev).await?;
        self.handle_action()?;
        self.auto_selector();
        self.files = FileManager::new(self);
        Ok(())
    }

    async fn receive_event(&mut self, ev: &mut EventThread) -> Result<(), Box<dyn Error>> {
        if let Ok(event) = ev.read() {
            self.handle_dialog(&event)?;
            if let Event::Key(event) = event {
                if event.kind == KeyEventKind::Press {
                    self.handle_keys(event);
                }
                if self.quit {
                    ev.shatdown().await?;
                } else {
                    ev.respond().await?;
                }
            }
        }
        Ok(())
    }

    async fn open_editor(&mut self) -> io::Result<()> {
        if let Some(file) = self.cur_file() {
            shell::editor(file).await?;
        }
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;
        self.editor = false;
        Ok(())
    }

    pub fn cur_file(&self) -> Option<&PathBuf> {
        self.files.require(self.cursor)
    }

    pub fn menu_opened(&self) -> bool {
        self.menu.is_some()
    }

    pub fn is_search(&self) -> bool {
        self.finder.is_some()
    }
}
