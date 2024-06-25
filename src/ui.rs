use crate::{action::Action, app::App, event::Signal, shell};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::{error::Error, io};
use tokio::sync::mpsc::Sender;
use tui_input::{backend::crossterm as backend, Input};
use unicode_segmentation::UnicodeSegmentation;

impl App {
    pub async fn render_mode<F: FnMut(&mut App) -> Result<(), Box<dyn Error>>>(
        &mut self,
        mut looper: F,
        sender: &Sender<Signal>,
    ) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)?;

        loop {
            if self.editor {
                sender.send(Signal::Pause).await?;
                shell::nvim(&self.files[self.cursor]).await?;
                sender.send(Signal::Pause).await?;
                execute!(io::stdout(), EnterAlternateScreen, Hide)?;
                self.editor = false;
            } else {
                self.ui()?;
                looper(self)?;
                if self.quit {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen, Show)?;
        Ok(())
    }

    fn ui(&self) -> Result<(), Box<dyn Error>> {
        let (cols, rows) = terminal::size()?;
        let path = self.path.to_str().unwrap_or("/");
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
        )?;

        let color = if !self.selected.is_empty() {
            Color::DarkBlue
        } else if self.dialog.is_some() {
            Color::Green
        } else {
            Color::Grey
        };
        render_line((cols, 1), color)?;
        render_line((cols, rows - 2), color)?;

        let buf = page - 1;
        let buf = buf * max;
        for p in 0..rows - 4 {
            let i = p as usize;
            execute!(io::stdout(), MoveTo(0, p + 2))?;
            if self.files.len() >= buf && self.files.len() - buf > i {
                let file = &self.files[i + buf];
                let file_names = crate::filename(&file).chars().take(65).collect::<String>();
                let file_len = file_names.graphemes(true).count();
                let pad = (file_names.len() - file_len) / 2;
                let select = if self.selected.contains(&i) {
                    Color::Rgb {
                        r: 100,
                        g: 100,
                        b: 100,
                    }
                } else {
                    Color::Reset
                };
                let mod_time = if let Ok(meta) = file.metadata() {
                    let datetime = DateTime::<Local>::from(meta.modified()?);
                    datetime.format("%Y/%m/%d %H:%M").to_string()
                } else {
                    String::from("       N/A      ")
                };
                execute!(
                    io::stdout(),
                    SetBackgroundColor(select),
                    Print(if self.cursor == i + buf { "> " } else { "  " }),
                    Print(" ▎ "),
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
                    Print(" ".repeat(cols as usize - file_len - pad - mod_time.len() - 9)),
                    Print("▎ "),
                    Print(mod_time),
                    Print(" ▎"),
                )?;
            } else {
                execute!(io::stdout(), ResetColor, Print(" ".repeat(cols as usize)))?;
            }
        }

        Ok(())
    }
}

fn render_line((cols, rows): (u16, u16), color: Color) -> Result<(), Box<dyn Error>> {
    execute!(
        io::stdout(),
        MoveTo(0, rows),
        SetBackgroundColor(color),
        Clear(ClearType::CurrentLine),
        Print(" ".repeat(cols as usize)),
        ResetColor
    )?;
    Ok(())
}

pub struct Dialog {
    pub input: Input,
    pub action: Action,
}

impl From<Action> for Dialog {
    fn from(value: Action) -> Self {
        Dialog {
            action: value,
            input: "".into(),
        }
    }
}

impl Dialog {
    pub fn write_backend<S: AsRef<str>>(&self, text: S) -> io::Result<()> {
        let text = text.as_ref();
        execute!(
            io::stdout(),
            MoveTo(1, 40),
            Clear(ClearType::CurrentLine),
            Print(text)
        )?;
        backend::write(
            &mut io::stdout(),
            self.input.value(),
            self.input.cursor(),
            ((text.len() + 2) as u16, 40),
            30,
        )
    }
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
