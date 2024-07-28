use crate::App;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Color, Print, ResetColor},
    terminal,
};
use std::{error::Error, io};
use termkit::render;

pub mod dialog;
mod file_rows;
mod header;

pub use dialog::*;

impl App {
    pub fn ui(&self) -> Result<(), Box<dyn Error>> {
        let (cols, rows) = terminal::size()?;
        let path = self.path.to_str().unwrap_or("/");
        let len = self.finder.len();
        let max = (rows - 4) as usize;
        let page = (self.cursor / max) + 1;
        let page_size = (len + max - 1) / max;

        header::render(path, cols, (page, page_size))?;

        let color = self.bar_color();
        render::horizontal_bar(1, color)?;
        render::horizontal_bar(rows - 2, color)?;

        let buf = (page - 1) * max;
        for p in 0..rows - 4 {
            let i = p as usize;
            execute!(io::stdout(), MoveTo(0, p + 2))?;
            if let Some(file) = self.finder.require(i + buf) {
                file_rows::render(
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
            self.finder.is_search(),
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
