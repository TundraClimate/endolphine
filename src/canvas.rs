use crate::{canvas_cache, error::*};
use crossterm::{
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal,
};

#[macro_export]
macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if &crate::canvas_cache::get($row) != $tag && crate::app::get_row()? != 0 {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::app::get_view_shift(), $row),
                $($cmd),+,
                crossterm::style::ResetColor
            ).map_err(|_| crate::error::EpError::DisplayViewLineFailed)?;
        }
        crate::canvas_cache::insert($row, $tag.to_string());
    }};
}

pub fn render() -> EpResult<()> {
    let (cols, rows) = terminal::size().unwrap_or((100, 100));
    render_header(cols)?;

    render_footer(rows - 2, cols)?;
    Ok(())
}

fn colored_bar(color: Color, len: u16) -> String {
    format!(
        "{}{}{}",
        SetBackgroundColor(color),
        " ".repeat(len as usize),
        ResetColor
    )
}

fn render_header(bar_length: u16) -> EpResult<()> {
    di_view_line!("title", 0, Print("test"));
    di_view_line!(
        "header_bar",
        1,
        Print(colored_bar(Color::White, bar_length))
    );

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(Color::White, bar_length))
    );

    if !canvas_cache::contain_key(row + 1) {
        di_view_line!("empty", row + 1, Print(""));
    }

    Ok(())
}
