use crate::{app, canvas_cache, color, error::*, misc};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal, Command,
};
use si_scale::helpers;
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

#[macro_export]
macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if &crate::canvas_cache::get(($row, 0)) != &$tag && crate::app::get_row() != 0 {
            crate::canvas_cache::insert(($row, 0), $tag.to_string());
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::app::get_view_shift(), $row),
                crossterm::style::SetBackgroundColor(crate::canvas::app_bg()),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
                $($cmd),+,
                crossterm::style::ResetColor
            ).map_err(|_| crate::error::EpError::DisplayViewLineFailed)
        } else { Ok(()) }
    }};
}

pub fn app_bg() -> Color {
    if app::menu().is_enabled() {
        color::APP_BG_DARK
    } else {
        color::APP_BG
    }
}

pub fn render() -> EpResult<()> {
    let (cols, rows) = terminal::size().unwrap_or((100, 100));

    if rows <= 4 {
        return Ok(());
    }

    render_header(cols)?;

    render_body()?;

    render_footer(rows - 2, cols)?;

    render_menu()?;
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
    let filename = format!("{}/", misc::file_name(&current_path));

    let usr = option_env!("USER").map_or("/root".to_string(), |u| match u {
        "root" => "/root".to_string(),
        user => format!("/home/{}", user),
    });

    let parent = misc::parent(&current_path)
        .to_str()
        .map_or("_OsCompatible_".to_string(), |p| {
            let rep = p.replacen(&usr, "~", 1);
            match rep.as_str() {
                "/" => "".to_string(),
                replaced => format!("{}/", replaced),
            }
        });

    let pwd = format!(
        "{}{}{}",
        parent,
        SetForegroundColor(color::HEADER_CURRENT_PATH_ON_DARK),
        filename
    );

    di_view_line!(
        format!("{}", &filename),
        0,
        Print(format!(" {} in {}", filename, pwd))
    )?;

    let cursor = app::cursor();

    let page_size = app::get_row().saturating_sub(4);
    let page = cursor.current() / page_size as usize + 1;
    let len = misc::child_files_len(&app::get_path());

    let page_area = format!(
        "{}{} Page {} {}(All {} items)",
        SetBackgroundColor(color::DEFAULT_BAR),
        SetForegroundColor(color::HEADER_BAR_TEXT_DEFAULT),
        page,
        SetForegroundColor(color::HEADER_BAR_TEXT_LIGHT),
        len
    );

    di_view_line!(
        format!("{}{}", page, len),
        1,
        Print(colored_bar(color::DEFAULT_BAR, bar_length)),
        MoveTo(app::get_view_shift(), 1),
        Print(page_area),
    )?;

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(color::DEFAULT_BAR, bar_length))
    )?;

    if !canvas_cache::contain_key((row + 1, 0)) {
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
        let is_cursor_pos = cursor.current() == abs_i;

        if let Some(f) = pagenated.get(rel_i as usize) {
            render_file_line(rel_i, is_cursor_pos, f, cursor.is_selected(abs_i))?;
        } else {
            render_empty_line(rel_i)?;
        }
    }
    Ok(())
}

fn render_file_line(
    rel_i: u16,
    is_cursor_pos: bool,
    file: &PathBuf,
    is_selected: bool,
) -> EpResult<()> {
    let c = if is_cursor_pos { ">" } else { " " };
    let filename = colored_file_name(&file);
    let under_name_color = SetBackgroundColor(if is_selected {
        color::SELECTED
    } else if is_cursor_pos {
        color::UNDER_CURSOR
    } else {
        app_bg()
    });
    let bsize = colored_bsize(&file);
    let time = colored_last_modified(&file);
    let permission = format_permissions(permission(&file));
    di_view_line!(
        format!(
            "{}{}{}{}{}{}{}",
            rel_i, c, filename, under_name_color, permission, bsize, time
        ),
        rel_i + 2,
        Print(format!(
            "{} | {} {} {} {}{}{}",
            c,
            permission,
            bsize,
            time,
            under_name_color,
            filename,
            SetBackgroundColor(app_bg())
        )),
    )
}

fn render_empty_line(rel_i: u16) -> EpResult<()> {
    di_view_line!(format!("{}", rel_i), rel_i + 2, Print(""))
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

fn colored_file_name(path: &PathBuf) -> String {
    format!(
        "{}{}",
        SetForegroundColor(match path {
            path if !path.exists() => color::PATH_NAME_BROKEN,
            path if path.is_symlink() => color::PATH_NAME_SYMLINK,
            path if path.is_dir() => color::PATH_NAME_DIRECTORY,
            path if path.is_file() => color::PATH_NAME_FILE,
            _ => color::PATH_NAME_BROKEN,
        }),
        misc::file_name(path)
    )
}

fn colored_bsize(path: &PathBuf) -> String {
    let Ok(metadata) = path.symlink_metadata() else {
        return String::from("       x");
    };
    if metadata.is_dir() {
        "       -".into()
    } else {
        let size = metadata.len();

        format!("{:>8}", helpers::bytes1(size as f64))
    }
}

fn colored_last_modified(path: &PathBuf) -> String {
    let Ok(metadata) = path.symlink_metadata() else {
        return String::from("       x");
    };
    let Ok(modified) = metadata.modified() else {
        return String::from("       x");
    };
    let datetime: DateTime<Local> = DateTime::from(modified);

    format!(
        "{}{}",
        SetForegroundColor(color::LAST_MODIFIED_TIME),
        datetime.format("%y %m/%d %H:%M")
    )
}

fn permission(path: &PathBuf) -> Vec<char> {
    let Ok(metadata) = path.symlink_metadata() else {
        return "---------".chars().collect();
    };
    let mode = metadata.permissions().mode();

    let permissions = format!(
        "{}{}{}{}{}{}{}{}{}",
        if mode & 0o400 != 0 { "r" } else { "-" },
        if mode & 0o200 != 0 { "w" } else { "-" },
        if mode & 0o100 != 0 { "x" } else { "-" },
        if mode & 0o040 != 0 { "r" } else { "-" },
        if mode & 0o020 != 0 { "w" } else { "-" },
        if mode & 0o010 != 0 { "x" } else { "-" },
        if mode & 0o004 != 0 { "r" } else { "-" },
        if mode & 0o002 != 0 { "w" } else { "-" },
        if mode & 0o001 != 0 { "x" } else { "-" },
    );

    permissions.chars().collect()
}

fn format_permissions(permission: Vec<char>) -> String {
    permission
        .chunks(3)
        .enumerate()
        .map(|(i, chunk)| {
            let (read, write, exe) = (chunk.get(0), chunk.get(1), chunk.get(2));
            format!(
                "{}{}{}",
                fpermission(read, i * 3),
                fpermission(write, i * 3 + 1),
                fpermission(exe, i * 3 + 2)
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn fpermission(permission: Option<&char>, index: usize) -> String {
    let color = match index % 3 {
        0 => color::PERMISSION_READ,
        1 => color::PERMISSION_WRITE,
        _ => color::PERMISSION_EXE,
    };
    SetForegroundColor(color).to_string() + permission.unwrap_or(&'-').to_string().as_str()
}

#[macro_export]
macro_rules! log {
    ($text:expr) => {{
        let row = crate::app::get_row();
        let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
        crate::di_view_line!(
            format!("{}", ts),
            row - 1,
            crossterm::style::Print(format!("{} {}", ts, $text))
        )
    }};
}

#[macro_export]
macro_rules! di_menu_line {
    ($row:expr, $tag:expr, $text:expr) => {{
        if &crate::canvas_cache::get(($row, 1)) != &$tag && crate::app::get_row() != 0 {
            crate::canvas_cache::insert(($row, 1), $tag.to_string());
            let slide = crate::app::get_view_shift();
            let bg = menu_bg();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::style::SetBackgroundColor(bg),
                crate::canvas::OverWrite(slide, $row),
                crossterm::cursor::MoveTo(0, $row),
                crossterm::style::Print($text),
                crossterm::cursor::MoveTo(slide - 1, $row),
                crossterm::style::SetBackgroundColor(bg),
                crossterm::style::Print(']'),
            )
            .map_err(|_| EpError::DisplayMenuLineFailed)
        } else {
            Ok(())
        }
    }};
}

struct OverWrite(u16, u16);

impl Command for OverWrite {
    fn write_ansi(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "\x1B[{};{}H", self.1 + 1, 0)?;
        write!(f, "{}", " ".repeat(self.0 as usize))?;
        Ok(())
    }
}

fn menu_bg() -> Color {
    if app::menu().is_enabled() {
        color::MENU_BG
    } else {
        color::MENU_BG_DARK
    }
}

fn render_menu() -> EpResult<()> {
    let slide_len = app::get_view_shift();
    if slide_len == 0 {
        return Ok(());
    }

    let row = app::get_row();

    di_menu_line!(0, "title", format!(" Select to Move "))?;
    di_menu_line!(1, "sep", format!("{}", "-".repeat(slide_len as usize - 1)))?;

    let menu = app::menu();
    let elements = menu.elements();
    let cursor = menu.cursor().current() as u16;
    for i in 2..row {
        if let Some(element) = elements.get(i as usize - 2) {
            let tag = element
                .tag()
                .chars()
                .take(slide_len as usize - 1)
                .collect::<String>();
            let is_cursor_pos = i - 2 == cursor;
            let cur = if is_cursor_pos { ">" } else { " " };
            let under_name_color = SetBackgroundColor(if is_cursor_pos && menu.is_enabled() {
                color::MENU_UNDER_CURSOR
            } else {
                menu_bg()
            });
            di_menu_line!(
                i,
                format!("{}{}", cur, element.tag()),
                format!(
                    "{} |{} {} {}",
                    cur,
                    under_name_color,
                    tag,
                    SetBackgroundColor(menu_bg())
                )
            )?;
        } else {
            di_menu_line!(i, "empty", "")?;
        }
    }

    Ok(())
}
