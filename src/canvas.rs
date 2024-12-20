use crate::{app, canvas_cache, color, error::*, misc};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use std::path::PathBuf;

#[macro_export]
macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if &crate::canvas_cache::get($row) != &$tag && crate::app::get_row() != 0 {
            crate::canvas_cache::insert($row, $tag.to_string());
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::app::get_view_shift(), $row),
                crossterm::style::SetBackgroundColor(crate::color::APP_BG),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
                $($cmd),+,
                crossterm::style::ResetColor
            ).map_err(|_| crate::error::EpError::DisplayViewLineFailed)
        } else { Ok(()) }
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
    let current_path = app::get_path();
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
        )?;
    }

    let cursor = app::cursor();

    let page_size = app::get_row().saturating_sub(4);
    let page = cursor.current() / page_size as usize + 1;
    let len = misc::child_files(&app::get_path()).len();

    di_view_line!(
        format!("{}{}", page, len),
        1,
        Print(colored_bar(color::DEFAULT_BAR, bar_length)),
        MoveTo(app::get_view_shift(), 1),
        Print(format!(
            "{}{} Page {} {}(All {} items)",
            SetBackgroundColor(color::DEFAULT_BAR),
            SetForegroundColor(Color::Black),
            page,
            SetForegroundColor(Color::Black),
            len
        )),
    )?;

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(color::DEFAULT_BAR, bar_length))
    )?;

    if !canvas_cache::contain_key(row + 1) {
        di_view_line!("empty", row + 1, Print(""))?;
    }

    Ok(())
}

fn render_body() -> EpResult<()> {
    let page_size = app::get_row().saturating_sub(4);

    if page_size == 0 {
        return Ok(());
    }

    let path = app::get_path();
    let child_files = misc::sorted_child_files(&path);
    let cursor = app::cursor();
    let page = cursor.current() / page_size as usize + 1;
    let pagenated = pagenate(&child_files, page_size, page);
    for rel_i in 0..(app::get_row().saturating_sub(4)) {
        let abs_i = (page_size as usize * (page - 1)) + rel_i as usize;
        if let Some(f) = pagenated.get(rel_i as usize) {
            let c = if cursor.current() == abs_i { ">" } else { " " };
            let filename = misc::file_name(f);
            let selected = if cursor.is_selected(abs_i) { "]" } else { " " };
            di_view_line!(
                format!("{}{}{}{}", rel_i, c, filename, selected),
                rel_i + 2,
                Print(format!("{} | {} ", c, filename)),
                Print(selected)
            )?;
        } else {
            di_view_line!(format!("{}", rel_i), rel_i + 2, Print(""))?;
        }
    }
    Ok(())
}

fn pagenate(full: &Vec<PathBuf>, page_size: u16, current_page: usize) -> Vec<PathBuf> {
    if current_page == 0 {
        return vec![];
    }

    let chunks = full.chunks(page_size as usize).collect::<Vec<_>>();
    chunks
        .get(current_page - 1)
        .map(|p| p.to_vec())
        .unwrap_or(vec![])
}

#[macro_export]
macro_rules! log {
    ($text:expr) => {{
        let row = crate::app::get_row();
        crate::di_view_line!(
            format!("{}", chrono::Utc::now().timestamp_micros()),
            row - 1,
            crossterm::style::Print($text)
        )
    }};
}
