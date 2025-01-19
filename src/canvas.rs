use crate::{color, error::*, global, misc};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use si_scale::helpers;
use std::{os::unix::fs::PermissionsExt, path::PathBuf};

macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if !global::cache_match(($row, 0), &$tag) && global::get_height() != 0 {
            global::cache_insert(($row, 0), $tag.to_string());
            crossterm::execute!(
                std::io::stdout(),
                MoveTo(global::get_view_shift(), $row),
                SetBackgroundColor(color::app_bg()),
                Clear(ClearType::UntilNewLine),
                $($cmd),+,
                ResetColor
            ).map_err(|_| EpError::DisplayViewLineFailed)
        } else { Ok(()) }
    }};
}

macro_rules! di_menu_line {
    ($row:expr, $tag:expr, $($cmd:expr),+ $(,)?) => {{
        if !global::cache_match(($row, 1), &$tag) && global::get_height() != 0 {
            global::cache_insert(($row, 1), $tag.to_string());
            let slide = global::get_view_shift();
            let bg = color::menu_bg();
            crossterm::execute!(
                std::io::stdout(),
                SetBackgroundColor(bg),
                MoveTo(0, $row),
                Print(" ".repeat(slide as usize)),
                MoveTo(0, $row),
                $($cmd)+,
                MoveTo(slide - 1, $row),
                SetBackgroundColor(bg),
                Print(']'),
            )
            .map_err(|_| EpError::DisplayMenuLineFailed)
        } else {
            Ok(())
        }
    }};
}

#[macro_export]
macro_rules! log {
    ($text:expr) => {{
        let row = crate::global::get_height();
        if let Err(_) = crossterm::execute!(
            std::io::stdout(),
            crossterm::cursor::MoveTo(0, row),
            crossterm::style::Print($text),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
        ) {
            crate::error::EpError::Log.handle();
        };
    }};

    ($text:expr, $is_dbg:expr) => {{
        if $is_dbg {
            let row = crate::global::get_height();
            let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
            let ts = if $text == "" { " ".to_string() } else { ts };
            if let Err(_) = crossterm::execute!(
                std::io::stdout(),
                crossterm::cursor::MoveTo(0, row),
                crossterm::style::Print(format!("{} {}", ts, $text)),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::UntilNewLine),
            ) {
                crate::error::EpError::Log.handle();
            };
        } else {
            crate::log!($text);
        }
    }};
}

pub fn render() -> EpResult<()> {
    let (width, height) = (global::get_width(), global::get_height());

    if height <= 4 {
        return Ok(());
    }

    render_header(width)?;

    if height > 4 {
        render_body()?;
    }

    render_footer(height - 2, width)?;

    if width > 0 {
        render_menu()?;
    }

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

    let page = cursor.current() / misc::body_height() as usize + 1;
    let len = misc::child_files_len(&global::get_path());

    let page_area = format!(
        "{}{} Page {} {}(All {} items)",
        SetBackgroundColor(color::bar_color()),
        SetForegroundColor(color::HEADER_BAR_TEXT_DEFAULT),
        page,
        SetForegroundColor(color::HEADER_BAR_TEXT_LIGHT),
        len
    );

    di_view_line!(
        format!("{}{}", page, len),
        1,
        Print(colored_bar(color::bar_color(), bar_length)),
        MoveTo(global::get_view_shift(), 1),
        Print(page_area),
    )?;

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    di_view_line!(
        "footer_bar",
        row,
        Print(colored_bar(color::bar_color(), bar_length))
    )?;

    Ok(())
}

fn render_body() -> EpResult<()> {
    let height = misc::body_height();
    let cursor = global::cursor();
    let page = cursor.current() / height as usize + 1;
    let pagenated = pagenate(&misc::sorted_child_files(&global::get_path()), height, page);

    for rel_i in 0..height {
        let abs_i = (height as usize * (page - 1)) + rel_i as usize;
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

fn render_input_line(rel_i: u16) -> EpResult<()> {
    let Some(buf) = global::input_use(|i| i.buffer_load().clone()) else {
        return Ok(());
    };

    let buf: String = {
        let size = buf.chars().count();
        buf.chars().skip(size.saturating_sub(20)).collect()
    };

    crossterm::execute!(
        std::io::stdout(),
        MoveTo(global::get_view_shift() + 39, rel_i + 2),
        SetBackgroundColor(color::INPUT_BG),
        Print(" ".repeat(25)),
        MoveTo(global::get_view_shift() + 39, rel_i + 2),
        Print(buf),
        Print("▏"),
        ResetColor
    )
    .map_err(|_| EpError::DisplayViewLineFailed)?;

    Ok(())
}

fn render_file_line(
    rel_i: u16,
    is_cursor_pos: bool,
    file: &PathBuf,
    is_selected: bool,
) -> EpResult<()> {
    let body_row = BodyRow::new(file);
    let c = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(color::item_bg(is_selected, is_cursor_pos));
    di_view_line!(
        format!("{}{}{}{}", rel_i, body_row.gen_key(), c, under_name_color),
        rel_i + 2,
        Print(format!(
            "{} | {}{} {} {} {}{}{}",
            c,
            body_row.filetype,
            body_row.permission,
            body_row.bsize,
            body_row.time,
            under_name_color,
            body_row.filename,
            SetBackgroundColor(color::app_bg())
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

struct BodyRow {
    filename: String,
    filetype: String,
    bsize: String,
    time: String,
    permission: String,
}

impl BodyRow {
    fn new(path: &PathBuf) -> Self {
        Self {
            filename: Self::colored_file_name(path),
            filetype: Self::colored_file_type(path),
            bsize: Self::colored_bsize(path),
            time: Self::colored_last_modified(path),
            permission: Self::colored_permission(Self::format_permission(path)),
        }
    }

    fn gen_key(&self) -> String {
        format!(
            "{}{}{}{}{}",
            self.filename, self.filetype, self.bsize, self.time, self.permission
        )
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
            SetForegroundColor(color::path_name(path)),
            misc::file_name(path),
            if let Some(target) = Self::symlink_target(path) {
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

    fn format_permission(path: &PathBuf) -> Vec<char> {
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

    fn colored_permission(permission: Vec<char>) -> String {
        permission
            .chunks(3)
            .enumerate()
            .map(|(i, chunk)| {
                let (read, write, exe) = (chunk.get(0), chunk.get(1), chunk.get(2));
                format!(
                    "{}{}{}",
                    Self::colored_permission_element(read, i * 3),
                    Self::colored_permission_element(write, i * 3 + 1),
                    Self::colored_permission_element(exe, i * 3 + 2)
                )
            })
            .collect::<Vec<_>>()
            .join("")
    }

    fn colored_permission_element(permission: Option<&char>, index: usize) -> String {
        format!(
            "{}{}",
            SetForegroundColor(color::permission(index)),
            permission.unwrap_or(&'-')
        )
    }
}

fn render_menu() -> EpResult<()> {
    let slide_len = global::get_view_shift();
    if slide_len == 0 {
        return Ok(());
    }

    di_menu_line!(0, "title", Print(" Select to Cd "))?;
    di_menu_line!(1, "sep", Print("-".repeat(slide_len as usize - 1)))?;

    let menu = global::menu();
    let cursor = menu.cursor();

    for i in 2..misc::body_height() + 3 {
        if let Some(element) = menu.elements().get(i as usize - 2) {
            let is_cursor_pos = i as usize - 2 == cursor.current();

            render_menu_line(
                i,
                element.tag(),
                slide_len,
                is_cursor_pos,
                menu.is_enabled(),
            )?;
        } else {
            di_menu_line!(i, "empty", Print(""))?;
        }
    }

    Ok(())
}

fn render_menu_line(
    row: u16,
    tag: &str,
    slide_len: u16,
    is_cursor_pos: bool,
    menu_enabled: bool,
) -> EpResult<()> {
    let tag = tag.chars().take(slide_len as usize - 1).collect::<String>();
    let cur = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(color::menu_item_bg(is_cursor_pos, menu_enabled));

    di_menu_line!(
        row,
        format!("{}{}", cur, tag),
        Print(format!(
            "{} |{} {}{} {}{}",
            cur,
            under_name_color,
            SetForegroundColor(color::MENU_TAG_COLOR),
            tag,
            SetBackgroundColor(color::menu_bg()),
            ResetColor,
        ))
    )?;
    Ok(())
}
