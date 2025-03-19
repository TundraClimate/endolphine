use super::Widget;
use crate::{app, cursor, input, misc, theme};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use si_scale::helpers;
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

pub struct Body;

fn pagenate(full: &[PathBuf], page_size: u16, current_page: usize) -> Vec<PathBuf> {
    if current_page == 0 {
        return vec![];
    }

    let chunks = full.chunks(page_size as usize).collect::<Vec<_>>();
    chunks
        .get(current_page - 1)
        .map(|p| p.to_vec())
        .unwrap_or(vec![])
}

fn render_input(pos: (u16, u16), width: u16, padding: (u16, u16)) -> Result<(), app::Error> {
    let Some(buf) = input::use_f(|i| i.buffer_load().clone()) else {
        return Ok(());
    };

    let buf: String = {
        let size = buf.chars().count();
        let mut buf = buf
            .chars()
            .skip(size.saturating_sub(width as usize))
            .collect::<String>();
        buf.insert(input::use_f(|i| i.cursor_current()), '▏');
        buf
    };

    crossterm::queue!(
        std::io::stdout(),
        MoveTo(pos.0, pos.1),
        SetBackgroundColor(theme::scheme().input),
        Print(" ".repeat((padding.0 + width + padding.1) as usize)),
        MoveTo(pos.0 + padding.0, pos.1),
        Print(buf),
        ResetColor
    )
    .map_err(|_| app::Error::InRenderInput)?;

    Ok(())
}

fn render_input_line(rel_i: u16) -> Result<(), app::Error> {
    let name_col = 39;
    render_input((super::get_view_shift() + name_col, rel_i + 2), 20, (0, 5))?;

    Ok(())
}

fn render_file_line(
    rel_i: u16,
    is_cursor_pos: bool,
    file: &PathBuf,
    is_selected: bool,
) -> Result<(), app::Error> {
    let c = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(theme::item_bg(is_selected, is_cursor_pos));
    let body_row = BodyRow::new(file, c.into(), under_name_color);
    Body::cached_render_row(
        &format!("{}{}", rel_i, body_row.gen_key()),
        rel_i,
        body_row.to_string(),
    )
}

fn render_empty_line(rel_i: u16) -> Result<(), app::Error> {
    if rel_i == 0 {
        let row = format!(
            "{}> | Press 'a' to create the New file | Empty",
            SetForegroundColor(theme::bar_color()),
        );
        Body::cached_render_row(&rel_i.to_string(), rel_i, row)
    } else {
        Body::cached_render_row(&rel_i.to_string(), rel_i, "".to_string())
    }
}

struct BodyRow {
    cursor: String,
    filename: String,
    filetype: String,
    bsize: String,
    time: String,
    permission: String,
    under_name_color: SetBackgroundColor,
}

impl BodyRow {
    fn new(path: &PathBuf, cursor: String, under_name_color: SetBackgroundColor) -> Self {
        Self {
            cursor,
            filename: Self::colored_file_name(path),
            filetype: Self::colored_file_type(path),
            bsize: Self::colored_bsize(path),
            time: Self::colored_last_modified(path),
            permission: Self::colored_permission(Self::format_permission(path)),
            under_name_color,
        }
    }

    fn gen_key(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            self.cursor,
            self.filename,
            self.filetype,
            self.bsize,
            self.time,
            self.permission,
            self.under_name_color
        )
    }

    fn colored_file_type(path: &PathBuf) -> String {
        format!(
            "{}{}",
            SetForegroundColor(theme::scheme().perm_ty),
            match path {
                path if path.is_symlink() => 'l',
                path if path.is_dir() => 'd',
                path if path.is_file() => '-',
                _ => 'o',
            }
        )
    }

    fn surround_from_matcher(text: String) -> String {
        let mut pos = 0usize;
        let mut pat_len = 0usize;
        if app::is_match_grep(|m| {
            pat_len = m.len();
            !m.is_empty() && (text).find(m).inspect(|p| pos = *p).is_some()
        }) {
            let end_pos = pos + pat_len;
            let surround_color = SetBackgroundColor(theme::scheme().search_surround);
            let reset_color = SetBackgroundColor(theme::app_bg());
            format!(
                "{}{}{}{}{}",
                &text[..pos],
                surround_color,
                &text[pos..end_pos],
                reset_color,
                &text[end_pos..]
            )
        } else {
            text
        }
    }

    fn colored_file_name(path: &PathBuf) -> String {
        format!(
            "{}{}{}",
            SetForegroundColor(theme::path_name(path)),
            Self::surround_from_matcher(misc::file_name(path).to_owned()),
            if let Some(target) = Self::symlink_target(path) {
                format!(" -> {}", target)
            } else {
                "".into()
            }
        )
    }

    fn symlink_target(path: &Path) -> Option<String> {
        if !path.is_symlink() {
            return None;
        }

        if let Ok(link) = path.read_link() {
            Some(link.to_str().unwrap().into())
        } else {
            Some("Broken symlink".into())
        }
    }

    fn colored_bsize(path: &Path) -> String {
        let Ok(metadata) = path.symlink_metadata() else {
            return String::from("       x");
        };

        let bod = if metadata.is_dir() {
            "       -".into()
        } else {
            let size = metadata.len();

            helpers::bytes1(size as f64)
        };

        format!(
            "{}{:>8}",
            SetForegroundColor(theme::scheme().row_bsize),
            bod
        )
    }

    fn colored_last_modified(path: &Path) -> String {
        let Ok(metadata) = path.symlink_metadata() else {
            return String::from("       x");
        };
        let Ok(modified) = metadata.modified() else {
            return String::from("       x");
        };
        let datetime: DateTime<Local> = DateTime::from(modified);

        format!(
            "{}{}",
            SetForegroundColor(theme::scheme().row_mod_time),
            datetime.format("%y %m/%d %H:%M")
        )
    }

    fn format_permission(path: &Path) -> Vec<char> {
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
                let (read, write, exe) = (chunk.first(), chunk.get(1), chunk.get(2));
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
            SetForegroundColor(theme::permission(index)),
            permission.unwrap_or(&'-')
        )
    }
}

impl std::fmt::Display for BodyRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | {}{} {} {} {}{}{}",
            self.cursor,
            self.filetype,
            self.permission,
            self.bsize,
            self.time,
            self.under_name_color,
            self.filename,
            SetBackgroundColor(theme::app_bg())
        )
    }
}

impl Widget for Body {
    const ID: u8 = 1;

    fn render(_size: (u16, u16)) -> Result<(), app::Error> {
        let height = misc::body_height();
        let cursor = cursor::load();
        let page = cursor.current() / height as usize + 1;
        let pagenated = pagenate(&misc::sorted_child_files(&app::get_path()), height, page);

        for rel_i in 0..height {
            let abs_i = (height as usize * (page - 1)) + rel_i as usize;
            let is_cursor_pos = cursor.current() == abs_i;

            if is_cursor_pos && input::use_f(|i| i.is_enable()) {
                render_input_line(rel_i)?;
                continue;
            }

            if let Some(f) = pagenated.get(rel_i as usize) {
                render_file_line(rel_i, is_cursor_pos, f, cursor::is_selected(abs_i))?;
            } else {
                render_empty_line(rel_i)?;
            }
        }
        Ok(())
    }

    fn render_row(row: u16, cmds: String) -> std::io::Result<()> {
        crossterm::queue!(
            std::io::stdout(),
            MoveTo(super::get_view_shift(), row + 2),
            SetForegroundColor(theme::app_fg()),
            SetBackgroundColor(theme::app_bg()),
            Clear(ClearType::UntilNewLine),
            Print(cmds),
            ResetColor
        )
    }
}
