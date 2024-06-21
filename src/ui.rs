use crate::{actions::Action, app::App, event::Signal, shell};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::{backend::CrosstermBackend, prelude::*, terminal::Terminal};
use std::{error::Error, io};
use tokio::sync::mpsc::Sender;
use tui_input::{backend::crossterm as backend, Input};

impl App {
    pub async fn render_mode<F: FnMut(&mut App) -> bool>(
        &mut self,
        mut looper: F,
        sender: &Sender<Signal>,
    ) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

        loop {
            if self.editor {
                sender.send(Signal::Pause).await?;
                shell::nvim(self.files[self.cursor].clone()).await;
                sender.send(Signal::Pause).await?;
                execute!(io::stdout(), EnterAlternateScreen)?;
                self.editor = false;
            } else {
                terminal.draw(|f| self.ui(f))?;
                if looper(self) {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn ui(&self, frame: &mut Frame) {
        let (cols, rows) = terminal::size().unwrap();
        let path = self.path.to_str().unwrap();
        let len = self.files.len();
        let max = (rows - 4) as usize;
        let page = (self.cursor / max) + 1;
        let page_size = if len % max == 0 {
            len / max
        } else {
            (len / max) + 1
        };
        execute!(
            io::stdout(),
            MoveTo(2, 0),
            Print(path),
            Print(" ".repeat(cols as usize - path.len())),
            MoveTo(cols - 16, 0),
            Print(format!("page {} / {}", page, page_size))
        )
        .unwrap();

        execute!(
            io::stdout(),
            MoveTo(0, 1),
            SetBackgroundColor(Color::Grey),
            Clear(ClearType::CurrentLine),
            Print(" ".repeat(cols as usize)),
            ResetColor
        )
        .unwrap();

        let buf = page - 1;
        let buf = buf * max;
        for p in 0..rows - 4 {
            let i = p as usize;
            execute!(io::stdout(), MoveTo(0, p + 2)).unwrap();
            if self.files.len() >= buf && self.files.len() - buf > i {
                let file = &self.files[i + buf];
                let file_names = crate::filename(&file).chars().take(65).collect::<String>();
                let select = if self.selected.contains(&i) {
                    Color::Rgb {
                        r: 100,
                        g: 100,
                        b: 100,
                    }
                } else {
                    Color::Reset
                };
                let modified = file.metadata().unwrap().modified().unwrap();
                let datetime = DateTime::<Local>::from(modified);
                let mod_time = datetime.format("%Y/%m/%d %H:%M").to_string();
                execute!(
                    io::stdout(),
                    SetBackgroundColor(select),
                    Print(if self.cursor == i + buf { "> " } else { "  " }),
                    Print(" | "),
                    SetForegroundColor(if file.is_symlink() {
                        Color::Magenta
                    } else if file.is_dir() {
                        Color::Green
                    } else if file.is_file() {
                        Color::Yellow
                    } else {
                        Color::Red
                    }),
                    Print(&file_names),
                    ResetColor,
                    SetBackgroundColor(select),
                    Print(" ".repeat(cols as usize - file_names.len() - mod_time.len() - 9)),
                    Print("| "),
                    Print(mod_time),
                    Print(" |"),
                )
                .unwrap();
            } else {
                execute!(io::stdout(), ResetColor, Print(" ".repeat(cols as usize))).unwrap();
            }
        }

        execute!(
            io::stdout(),
            MoveTo(0, rows - 2),
            SetBackgroundColor(Color::Grey),
            Clear(ClearType::CurrentLine),
            Print(" ".repeat(cols as usize)),
            ResetColor
        )
        .unwrap();
    }
}

pub struct Dialog {
    pub input: Input,
    pub action: Action,
}

pub fn write_backend(dialog: &Dialog, text: &str) -> io::Result<()> {
    execute!(
        io::stdout(),
        MoveTo(0, 40),
        Clear(ClearType::CurrentLine),
        Print(text)
    )
    .unwrap();
    backend::write(
        &mut io::stdout(),
        dialog.input.value(),
        dialog.input.cursor(),
        ((text.len() + 1) as u16, 40),
        30,
    )
}

pub fn log(text: String) -> io::Result<()> {
    execute!(
        io::stdout(),
        MoveTo(1, 40),
        Clear(ClearType::CurrentLine),
        Print(text)
    )?;
    Ok(())
}
