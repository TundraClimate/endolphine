use super::Rect;
use crate::canvas;
use std::path::{Path, PathBuf};

pub(super) struct Viewer {
    wd: PathBuf,
    cursor_pos: usize,
    selection: Vec<usize>,
    grep: String,
    input_tag: Option<String>,
    input_buf: Option<String>,
    input_cursor: usize,
}

impl Viewer {
    pub(super) const ID: u8 = 3;

    pub(super) fn new(
        wd: PathBuf,
        cursor_pos: usize,
        selection: Vec<usize>,
        grep: String,
        input_tag: Option<String>,
        input_buf: Option<String>,
        input_cursor: usize,
    ) -> Self {
        Self {
            wd,
            cursor_pos,
            selection,
            grep,
            input_tag,
            input_buf,
            input_cursor,
        }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use crate::misc;
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.wd.hash(&mut hasher);
        self.cursor_pos.hash(&mut hasher);
        self.selection.hash(&mut hasher);
        self.grep.hash(&mut hasher);
        self.input_tag.hash(&mut hasher);
        self.input_buf.hash(&mut hasher);
        self.input_cursor.hash(&mut hasher);

        misc::child_files(&self.wd).hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crate::misc;

        let wd = &self.wd;
        let page_size = rect.height as usize;

        if page_size == 0 {
            return;
        }

        let page_index = self.cursor_pos / page_size;
        let items = pagenate(wd, page_size, page_index).unwrap_or_default();

        let (tag, ctx) = self
            .input_tag
            .as_ref()
            .and_then(|tag| tag.split_once(":"))
            .unzip();

        let is_input_active = tag == Some("RenameThisItem");
        let input_idx = ctx.and_then(|ctx| ctx.parse::<usize>().ok());

        for rel_i in 0..page_size {
            let abs_i = rel_i + page_size * page_index;

            match items.get(rel_i) {
                Some(_) if is_input_active && input_idx == Some(abs_i) => {
                    if let Some(ref input_buf) = self.input_buf {
                        render_input_row(rect, rel_i, input_buf, self.input_cursor);
                    }
                }
                Some(item) if &misc::entry_name(item) == ".ep.ed" => {
                    if let Some(ref input_buf) = self.input_buf {
                        render_input_row(rect, rel_i, input_buf, self.input_cursor);
                    }
                }
                Some(item) => render_item_row(
                    rect,
                    rel_i,
                    item,
                    self.cursor_pos == abs_i,
                    self.selection.contains(&abs_i),
                    &self.grep,
                ),
                None => render_empty_row(rect, rel_i),
            }
        }
    }
}

fn pagenate(wd: &Path, page_size: usize, page_index: usize) -> Option<Vec<PathBuf>> {
    use crate::misc;

    let usable_paths = misc::sorted_child_files(wd).to_vec();

    let pages = usable_paths.chunks(page_size).collect::<Vec<_>>();

    pages.get(page_index).map(|paths| paths.to_vec())
}

fn render_item_row(
    rect: Rect,
    index: usize,
    item: &Path,
    is_cursor_pos: bool,
    is_selected: bool,
    grep: &str,
) {
    use crate::{config, misc};
    use chrono::{DateTime, Local};
    use crossterm::style::{SetBackgroundColor, SetForegroundColor};
    use regex::Regex;
    use si_scale::helpers;
    use std::os::unix::fs::PermissionsExt;

    let theme = &config::get().theme;
    let cursor = if is_cursor_pos { ">" } else { " " };
    let file_type = format!(
        "{}{}",
        SetForegroundColor(theme.perm_ty.into()),
        match item {
            item if item.is_symlink() => 'l',
            item if item.is_dir() => 'd',
            item if item.is_file() => '-',
            _ => 'o',
        }
    );

    let Ok(metadata) = item.symlink_metadata() else {
        canvas::printin(
            rect,
            (0, index as u16),
            format!(
                "{}{} | Permission denied{}",
                SetForegroundColor(theme.item_broken.into()),
                SetBackgroundColor(theme.app_bg.into()),
                " ".repeat(rect.width.into())
            ),
        );

        return;
    };

    let mode = metadata.permissions().mode();
    let perm = [0, 3, 6]
        .into_iter()
        .flat_map(|range_shift| {
            [0, 1, 2].map(|perm_shift| {
                format!(
                    "{}{}",
                    [
                        SetForegroundColor(theme.perm_r.into()),
                        SetForegroundColor(theme.perm_w.into()),
                        SetForegroundColor(theme.perm_x.into())
                    ][perm_shift],
                    if mode & 0o400 >> range_shift >> perm_shift != 0 {
                        ['r', 'w', 'x'][perm_shift]
                    } else {
                        '-'
                    }
                )
            })
        })
        .collect::<String>();
    let bsize = format!(
        "{}{:>8}",
        SetForegroundColor(theme.item_parts_bsize.into()),
        if metadata.is_dir() {
            "       -".into()
        } else {
            let size = metadata.len();

            helpers::bytes1(size as f64)
        }
    );
    let lmd = format!(
        "{}{}",
        SetForegroundColor(theme.item_parts_lmd.into()),
        metadata
            .modified()
            .map(|sys_time| {
                DateTime::<Local>::from(sys_time)
                    .format("%y %m/%d %H:%M")
                    .to_string()
            })
            .unwrap_or(String::from("       x"))
    );
    let under_name = if is_selected {
        theme.item_bg_select
    } else if is_cursor_pos {
        theme.item_bg_cursor
    } else {
        theme.app_bg
    };
    let file_name = format!(
        "{}{}{}",
        SetForegroundColor(
            match item {
                path if !path.exists() => theme.item_broken,
                path if path.is_symlink() => theme.item_symlink,
                path if path.is_dir() => theme.item_dir,
                path if path.is_file() => theme.item_file,
                _ => theme.item_broken,
            }
            .into()
        ),
        'n: {
            let reg = Regex::new(grep);
            let name = misc::entry_name(item);

            if grep.is_empty() {
                break 'n name;
            }

            let Ok(regex) = reg else {
                break 'n name;
            };

            match regex.find(&name).map(|r| (r.start(), r.end())) {
                Some((start, end)) => {
                    let surround_color = SetBackgroundColor(theme.search_surround.into());
                    let reset_color = SetBackgroundColor(theme.app_bg.into());

                    format!(
                        "{}{}{}{}{}",
                        &name[..start],
                        surround_color,
                        &name[start..end],
                        reset_color,
                        &name[end..]
                    )
                }
                None => name,
            }
        },
        match item.read_link() {
            Ok(link) => format!(" -> {}", link.to_string_lossy()),
            Err(_) => "".to_string(),
        }
    );

    canvas::printin(
        rect,
        (0, index as u16),
        format!(
            "{}{}{} | {}{} {} {} {}{}{}{}",
            SetBackgroundColor(theme.app_bg.into()),
            SetForegroundColor(theme.app_fg.into()),
            cursor,
            file_type,
            perm,
            bsize,
            lmd,
            SetBackgroundColor(under_name.into()),
            file_name,
            SetBackgroundColor(theme.app_bg.into()),
            " ".repeat(rect.width.into())
        ),
    );
}

fn render_empty_row(rect: Rect, index: usize) {
    use crate::config;
    use crossterm::style::{SetBackgroundColor, SetForegroundColor};

    let theme = &config::get().theme;

    if index == 0 {
        let empty_msg = format!(
            "{}{}> | Press 'a' to create the New file | Empty{}",
            SetBackgroundColor(theme.app_bg.into()),
            SetForegroundColor(theme.bar_fg.into()),
            " ".repeat(rect.width.into())
        );

        canvas::printin(rect, (0, 0), empty_msg);
    } else {
        canvas::printin(
            rect,
            (0, index as u16),
            format!(
                "{}{}",
                SetBackgroundColor(theme.app_bg.into()),
                " ".repeat(rect.width.into())
            ),
        )
    }
}

fn render_input_row(rect: Rect, index: usize, input_buf: &str, input_cursor: usize) {
    use crate::config;
    use crossterm::style::{ResetColor, SetBackgroundColor, SetForegroundColor};

    let theme = &config::get().theme;
    let mut input = input_buf.to_string();

    input.insert(input_cursor, '▏');
    input.push_str(&" ".repeat(13usize.saturating_sub(input_buf.len())));

    let perm = (0..3)
        .flat_map(|_| {
            [0, 1, 2].map(|perm_shift| {
                format!(
                    "{}?",
                    [
                        SetForegroundColor(theme.perm_r.into()),
                        SetForegroundColor(theme.perm_w.into()),
                        SetForegroundColor(theme.perm_x.into())
                    ][perm_shift],
                )
            })
        })
        .collect::<String>();

    canvas::printin(
        rect,
        (0, index as u16),
        format!(
            "{}{}> | {}?{} {}???????? {}?? ??/?? ??:?? {}{}",
            SetBackgroundColor(theme.app_bg.into()),
            SetForegroundColor(theme.app_fg.into()),
            SetForegroundColor(theme.perm_ty.into()),
            perm,
            SetForegroundColor(theme.item_parts_bsize.into()),
            SetForegroundColor(theme.item_parts_lmd.into()),
            ResetColor,
            input
        ),
    );
}
