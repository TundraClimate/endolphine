use crate::{action::Action, app::App};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::{error::Error, io, path::PathBuf};
use termkit::render;
use tui_input::{backend::crossterm as backend, Input};
use unicode_segmentation::UnicodeSegmentation;

impl App {
    pub fn ui(&self) -> Result<(), Box<dyn Error>> {
        let (cols, rows) = terminal::size()?;
        let path = self.path.to_str().unwrap_or("/");
        let len = self.files.len();
        let max = (rows - 4) as usize;
        let page = (self.cursor / max) + 1;
        let page_size = (len + max - 1) / max;
        let color = self.bar_color();

        render_header(path, cols, (page, page_size))?;

        render::horizontal_bar(1, color)?;
        render::horizontal_bar(rows - 2, color)?;

        let buf = (page - 1) * max;
        for p in 0..rows - 4 {
            let i = p as usize;
            execute!(io::stdout(), MoveTo(0, p + 2))?;
            if let Some(file) = self.files.require(i + buf) {
                render_row(
                    file,
                    self.menu_opened(),
                    self.cursor == i + buf,
                    self.selected.contains(&i),
                    cols,
                )?;
            } else {
                execute!(io::stdout(), ResetColor, Print(" ".repeat(cols as usize)))?;
            }
        }
        Ok(())
    }

    fn bar_color(&self) -> Color {
        match (
            self.selected.is_empty(),
            self.is_search,
            self.dialog.is_some(),
            self.menu.is_some(),
        ) {
            (false, ..) => Color::DarkBlue,
            (true, _, _, true) => Color::Yellow,
            (true, true, ..) => Color::Magenta,
            (true, false, true, ..) => Color::Green,
            _ => Color::Grey,
        }
    }
}

fn render_header(path: &str, cols: u16, (page, page_size): (usize, usize)) -> io::Result<()> {
    execute!(
        io::stdout(),
        MoveTo(2, 0),
        Print(path),
        Print(" ".repeat(cols as usize - path.len() - 16)),
        MoveTo(cols - 16, 0),
        Print(format!("page {} / {}  ", page, page_size))
    )?;
    Ok(())
}

fn render_row(
    file: &PathBuf,
    is_menu: bool,
    is_cursor: bool,
    is_selected: bool,
    cols: u16,
) -> io::Result<()> {
    let file_names = crate::filename(&file).chars().take(65).collect::<String>();
    let file_len = file_names.graphemes(true).count();
    let pad = (file_names.len() - file_len) / 2;
    let select = if is_selected {
        Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        }
    } else {
        Color::Reset
    };
    execute!(
        io::stdout(),
        SetBackgroundColor(select),
        Print(if is_cursor { "~>" } else { "  " }),
        Print(" | "),
        SetForegroundColor(colored_path(file)),
        Print(&file_names),
        ResetColor,
        SetBackgroundColor(select),
        Print(" ".repeat(cols as usize - file_len - pad - 27)),
        Print("| "),
        SetForegroundColor(Color::DarkBlue),
        Print(info_block(file, is_menu)?),
        ResetColor,
        Print(" |"),
        Print(if is_cursor { " <" } else { "  " }),
    )?;
    Ok(())
}

fn colored_path(file: &PathBuf) -> Color {
    if file.is_symlink() {
        Color::Magenta
    } else if file.is_dir() {
        Color::Green
    } else if file.is_file() {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn info_block(file: &PathBuf, is_menu: bool) -> io::Result<String> {
    Ok(if let Ok(meta) = file.metadata() {
        let datetime = DateTime::<Local>::from(meta.modified()?);
        datetime.format("%Y/%m/%d %H:%M").to_string()
    } else if is_menu {
        String::from(" Open to Select ")
    } else {
        String::from("       N/A      ")
    })
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
        execute!(io::stdout(), MoveTo(1, 40), Print(text))?;
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
