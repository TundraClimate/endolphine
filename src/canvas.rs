use crate::{app, canvas_cache, color, error::*, misc};
use crossterm::{
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::path::PathBuf;

#[macro_export]
macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if &crate::canvas_cache::get($row) != &$tag && crate::app::get_row()? != 0 {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::app::get_view_shift(), $row),
                crossterm::style::SetBackgroundColor(crate::color::APP_BG),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
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

    render_body()?;

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
    let current_path = app::get_path()?;
    {
        let filename = if current_path == PathBuf::from("/") {
            "/"
        } else {
            &format!("{}{}", misc::file_name(&current_path), "/")
        };

        let pwd = {
            let usr = option_env!("USER").unwrap_or("root");
            let usr = if usr == "root" {
                "/root"
            } else {
                &format!("/home/{}", usr)
            };
            let parent = misc::parent(&current_path);
            let mut parent = parent
                .to_str()
                .unwrap_or("*Invalid Name*")
                .replacen(usr, "~", 1);
            if parent != "/" {
                parent.push('/')
            } else {
                parent.pop();
            }
            format!(
                "{}{}{}",
                parent,
                SetForegroundColor(color::HEADER_CURRENT_PATH_ON_DARK),
                filename
            )
        };

        di_view_line!(
            format!("{}", &filename),
            0,
            Print(format!(" {} in {}", filename, pwd))
        );
    }

    di_view_line!(
        "header_bar",
        1,
        Print(colored_bar(color::DEFAULT_BAR, bar_length))
    );

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(color::DEFAULT_BAR, bar_length))
    );

    if !canvas_cache::contain_key(row + 1) {
        di_view_line!("empty", row + 1, Print(""));
    }

    Ok(())
}

fn render_body() -> EpResult<()> {
    let path = app::get_path()?;
    let child_files = misc::sorted_child_files(&path);
    for rel_i in 0..(app::get_row()? - 4) {
        if let Some(f) = child_files.get(rel_i as usize) {
            di_view_line!(format!("{}", rel_i), rel_i + 2, Print(misc::file_name(f)))
        } else {
            di_view_line!(format!("{}", rel_i), rel_i + 2, Print(""))
        }
    }
    Ok(())
}

#[macro_export]
macro_rules! log {
    ($text:expr) => {{
        let row = crate::app::get_row()?;
        crate::di_view_line!(
            format!("{}", chrono::Utc::now().timestamp_micros()),
            row - 1,
            crossterm::style::Print($text)
        );
    }};
}
