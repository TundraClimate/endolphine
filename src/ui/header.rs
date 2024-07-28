use crossterm::{cursor::MoveTo, execute, style::Print};
use std::io;

pub fn render(path: &str, cols: u16, (page, page_size): (usize, usize)) -> io::Result<()> {
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
