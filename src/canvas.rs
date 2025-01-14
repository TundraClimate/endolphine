use crate::{color, error::*, global, misc};
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
        if &crate::canvas_cache::get(($row, 0)) != &$tag && crate::global::get_row() != 0 {
            crate::canvas_cache::insert(($row, 0), $tag.to_string());
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::global::get_view_shift(), $row),
                crossterm::style::SetBackgroundColor(crate::canvas::app_bg()),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
                $($cmd),+,
                crossterm::style::ResetColor
            ).map_err(|_| crate::error::EpError::DisplayViewLineFailed)
        } else { Ok(()) }
    }};
}

fn app_bg() -> Color {
    if global::menu().is_enabled() {
        color::APP_BG_DARK
    } else {
        color::APP_BG
    }
}

fn bar_color() -> Color {
    if global::menu().is_enabled() {
        color::BAR_DARK
    } else {
        color::BAR
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
    let current_path = global::get_path();
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

    let cursor = global::cursor();

    let page_size = global::get_row().saturating_sub(4);
    let page = cursor.current() / page_size as usize + 1;
    let len = misc::child_files_len(&global::get_path());

    let page_area = format!(
        "{}{} Page {} {}(All {} items)",
        SetBackgroundColor(bar_color()),
        SetForegroundColor(color::HEADER_BAR_TEXT_DEFAULT),
        page,
        SetForegroundColor(color::HEADER_BAR_TEXT_LIGHT),
        len
    );

    di_view_line!(
        format!("{}{}", page, len),
        1,
        Print(colored_bar(bar_color(), bar_length)),
        MoveTo(global::get_view_shift(), 1),
        Print(page_area),
    )?;

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(bar_color(), bar_length))
    )?;

    Ok(())
}

fn render_body() -> EpResult<()> {
    let page_size = global::get_row().saturating_sub(4);

    if page_size == 0 {
        return Ok(());
    }

    let path = global::get_path();
    let child_files = misc::sorted_child_files(&path);
    let cursor = global::cursor();
    let page = cursor.current() / page_size as usize + 1;
    let pagenated = pagenate(&child_files, page_size, page);

    for rel_i in 0..(global::get_row().saturating_sub(4)) {
        let abs_i = (page_size as usize * (page - 1)) + rel_i as usize;
        let is_cursor_pos = cursor.current() == abs_i;

        if is_cursor_pos && global::input_use(|i| i.is_enable()) {
            render_input_line(rel_i)?;
            continue;
        }

        if let Some(f) = pagenated.get(rel_i as usize) {
            render_file_line(rel_i, is_cursor_pos, f, cursor.is_selected(abs_i))?;
        } else {
            render_empty_line(rel_i)?;
        }
    }
    Ok(())
}

macro_rules! di_input_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if &crate::canvas_cache::get(($row, 0)) != &$tag && crate::global::get_row() != 0 {
            crate::canvas_cache::insert(($row, 0), $tag.to_string());
            crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(crate::global::get_view_shift() + 39, $row),
                crossterm::style::SetBackgroundColor(crate::color::INPUT_BG),
                crossterm::style::Print(" ".repeat(25)),
                crossterm::cursor::MoveTo(crate::global::get_view_shift() + 39, $row),
                $($cmd),+,
                crossterm::style::Print("▏"),
                crossterm::style::ResetColor
            ).map_err(|_| crate::error::EpError::DisplayViewLineFailed)
        } else { Ok(()) }
    }};
}

fn render_input_line(rel_i: u16) -> EpResult<()> {
    let Some(buf) = global::input_use(|i| i.buffer_load().clone()) else {
        return Ok(());
    };

    let buf: String = {
        let size = buf.chars().count();
        buf.chars().skip(size.saturating_sub(20)).collect()
    };

    di_input_line!(format!("input{}", buf), rel_i + 2, Print(buf))?;

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
    let filetype = colored_file_type(file);
    let bsize = colored_bsize(&file);
    let time = colored_last_modified(&file);
    let permission = format_permissions(permission(&file));
    di_view_line!(
        format!(
            "{}{}{}{}{}{}{}{}",
            rel_i, c, filename, filetype, under_name_color, permission, bsize, time
        ),
        rel_i + 2,
        Print(format!(
            "{} | {}{} {} {} {}{}{}",
            c,
            filetype,
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

fn colored_file_type(path: &PathBuf) -> String {
    format!(
        "{}{}",
        SetForegroundColor(color::PERMISSION_TYPE),
        match path {
            path if path.is_symlink() => 'l',
            path if path.is_dir() => 'd',
            path if path.is_file() => '-',
            _ => 'o',
        }
    )
}

fn colored_file_name(path: &PathBuf) -> String {
    format!(
        "{}{}{}",
        SetForegroundColor(match path {
            path if !path.exists() => color::PATH_NAME_BROKEN,
            path if path.is_symlink() => color::PATH_NAME_SYMLINK,
            path if path.is_dir() => color::PATH_NAME_DIRECTORY,
            path if path.is_file() => color::PATH_NAME_FILE,
            _ => color::PATH_NAME_BROKEN,
        }),
        misc::file_name(path),
        if let Some(target) = symlink_target(path) {
            format!(" -> {}", target)
        } else {
            "".into()
        }
    )
}

fn symlink_target(path: &PathBuf) -> Option<String> {
    if !path.is_symlink() {
        return None;
    }

    if let Ok(link) = path.read_link() {
        Some(link.to_str().unwrap().into())
    } else {
        Some("Broken symlink".into())
    }
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
        let row = crate::global::get_row();
        let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
        let ts = if $text == "" { " ".to_string() } else { ts };
        crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveTo(0, row),
            crossterm::style::Print(format!("{} {}", ts, $text)),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
        )
        .map_err(|_| crate::error::EpError::Log)
    }};
}

#[macro_export]
macro_rules! di_menu_line {
    ($row:expr, $tag:expr, $text:expr) => {{
        if &crate::canvas_cache::get(($row, 1)) != &$tag && crate::global::get_row() != 0 {
            crate::canvas_cache::insert(($row, 1), $tag.to_string());
            let slide = crate::global::get_view_shift();
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
    if global::menu().is_enabled() {
        color::MENU_BG
    } else {
        color::MENU_BG_DARK
    }
}

fn render_menu() -> EpResult<()> {
    let slide_len = global::get_view_shift();
    if slide_len == 0 {
        return Ok(());
    }

    let row = global::get_row();

    di_menu_line!(0, "title", " Select to Cd ")?;
    di_menu_line!(1, "sep", "-".repeat(slide_len as usize - 1))?;

    let menu = global::menu();
    let elements = menu.elements();
    let cursor = menu.cursor().current() as u16;
    for i in 2..row - 1 {
        if let Some(element) = elements.get(i as usize - 2) {
            render_menu_line(
                i,
                element.tag(),
                slide_len,
                i - 2 == cursor,
                menu.is_enabled(),
            )?;
        } else {
            di_menu_line!(i, "empty", "")?;
        }
    }

    Ok(())
}

fn render_menu_line(
    row: u16,
    tag: String,
    slide_len: u16,
    is_cursor_pos: bool,
    menu_enabled: bool,
) -> EpResult<()> {
    let tag = tag.chars().take(slide_len as usize - 1).collect::<String>();
    let cur = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(if is_cursor_pos && menu_enabled {
        color::MENU_UNDER_CURSOR
    } else {
        menu_bg()
    });

    di_menu_line!(
        row,
        format!("{}{}", cur, tag),
        format!(
            "{} |{} {}{} {}{}",
            cur,
            under_name_color,
            SetForegroundColor(color::MENU_TAG_COLOR),
            tag,
            SetBackgroundColor(menu_bg()),
            ResetColor,
        )
    )?;
    Ok(())
}
