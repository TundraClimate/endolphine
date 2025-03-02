use crate::{app, cursor, error::*, global, input, menu, misc, theme};
use chrono::{DateTime, Local};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use si_scale::helpers;
use std::{
    collections::HashMap,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicU16, Ordering},
    },
};

macro_rules! di_view_line {
    ($tag:expr, $row:expr, $($cmd:expr),+ $(,)?) => {{
        if !cache_match(($row, 0), &$tag) {
            cache_insert(($row, 0), $tag.to_string());
            crossterm::queue!(
                std::io::stdout(),
                MoveTo(get_view_shift(), $row),
                SetForegroundColor(theme::app_fg()),
                SetBackgroundColor(theme::app_bg()),
                Clear(ClearType::UntilNewLine),
                $($cmd),+,
                ResetColor
            ).map_err(|_| EpError::Display)
        } else { Ok(()) }
    }};
}

macro_rules! di_menu_line {
    ($row:expr, $tag:expr, $($cmd:expr),+ $(,)?) => {{
        if !cache_match(($row, 1), &$tag)  {
            cache_insert(($row, 1), $tag.to_string());
            let slide = get_view_shift();
            let bg = theme::widget_bg();
            crossterm::queue!(
                std::io::stdout(),
                SetForegroundColor(theme::widget_fg()),
                SetBackgroundColor(bg),
                MoveTo(0, $row),
                Print(" ".repeat(slide as usize)),
                MoveTo(0, $row),
                $($cmd)+,
                MoveTo(slide - 1, $row),
                SetBackgroundColor(theme::wid_bar_color()),
                SetForegroundColor(theme::scheme().label),
                Print("|"),
                SetBackgroundColor(bg),
                SetForegroundColor(theme::widget_fg()),
            )
            .map_err(|_| EpError::Display)
        } else {
            Ok(())
        }
    }};
}

#[macro_export]
macro_rules! log {
    ($text:expr) => {{
        use crossterm::cursor;
        use crossterm::style;
        use crossterm::terminal;
        use crossterm::terminal::ClearType;
        use std::io;
        let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
        if let Err(_) = crossterm::execute!(
            io::stdout(),
            style::ResetColor,
            cursor::MoveTo(0, row),
            style::Print($text),
            terminal::Clear(ClearType::UntilNewLine),
        ) {
            $crate::error::EpError::Display.handle()
        };
    }};

    ($text:expr, $is_dbg:expr) => {{
        if $is_dbg {
            use crossterm::cursor;
            use crossterm::style;
            use crossterm::terminal;
            use crossterm::terminal::ClearType;
            use std::io;
            let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
            let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
            let ts = if $text == "" { " ".to_string() } else { ts };
            if let Err(_) = crossterm::execute!(
                io::stdout(),
                cursor::MoveTo(0, row),
                style::Print(format!("{} {}", ts, $text)),
                terminal::Clear(ClearType::UntilNewLine),
            ) {
                $crate::error::EpError::Display.handle()
            };
        } else {
            $crate::log!($text);
        }
    }};
}

global!(VIEW_SHIFT<AtomicU16>, || AtomicU16::new(0), {
    pub fn get_view_shift() -> u16 {
        VIEW_SHIFT.load(Ordering::Relaxed)
    }

    pub fn set_view_shift(new_value: u16) {
        VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
    }
});

global!(
    CACHE<RwLock<HashMap<(u16, u8), String>>>,
    || RwLock::new(HashMap::new()),
    {
        pub fn cache_insert(key: (u16, u8), tag: String) {
            CACHE.write().unwrap().insert(key, tag);
        }

        pub fn cache_match(key: (u16, u8), tag: &str) -> bool {
            CACHE.read().unwrap().get(&key).map(|c| c.as_ref()) == Some(tag)
        }

        pub fn cache_clear() {
            CACHE.write().unwrap().clear();
        }
    }
);

pub fn render() -> EpResult<()> {
    let (width, height) = crossterm::terminal::size().unwrap_or((0, 0));

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

    use std::io::Write;

    std::io::stdout()
        .flush()
        .map_err(|e| EpError::Flush(e.kind().to_string()))?;

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
        SetForegroundColor(theme::scheme().path_picked),
        filename
    );

    di_view_line!(
        format!("{}", &filename),
        0,
        Print(format!(" {} in {}", filename, pwd))
    )?;

    let cursor = cursor::master();

    let page = cursor.current() / misc::body_height() as usize + 1;
    let len = misc::child_files_len(&app::get_path());

    let page_area = format!(
        "{}{} Page {} {}(All {} items)",
        SetBackgroundColor(theme::bar_color()),
        SetForegroundColor(theme::scheme().bar_text),
        page,
        SetForegroundColor(theme::scheme().bar_text_light),
        len
    );

    di_view_line!(
        format!("{}{}", page, len),
        1,
        Print(colored_bar(theme::bar_color(), bar_length)),
        MoveTo(get_view_shift(), 1),
        Print(page_area),
    )?;

    Ok(())
}

fn render_footer(row: u16, bar_length: u16) -> EpResult<()> {
    let procs = app::procs();
    let bar_text = format!(
        "{}{} {} process running",
        SetBackgroundColor(theme::bar_color()),
        SetForegroundColor(theme::scheme().bar_text),
        procs
    );

    di_view_line!(
        format!("{}", procs),
        row,
        Print(colored_bar(theme::bar_color(), bar_length)),
        MoveTo(get_view_shift(), row),
        Print(bar_text)
    )?;

    Ok(())
}

fn render_body() -> EpResult<()> {
    let height = misc::body_height();
    let cursor = cursor::master();
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
            render_file_line(rel_i, is_cursor_pos, f, cursor.is_selected(abs_i))?;
        } else {
            render_empty_line(rel_i)?;
        }
    }
    Ok(())
}

fn render_input(pos: (u16, u16), width: u16, padding: (u16, u16)) -> EpResult<()> {
    let Some(buf) = input::use_f(|i| i.buffer_load().clone()) else {
        return Ok(());
    };

    let buf: String = {
        let size = buf.chars().count();
        buf.chars()
            .skip(size.saturating_sub(width as usize))
            .collect()
    };

    crossterm::queue!(
        std::io::stdout(),
        MoveTo(pos.0, pos.1),
        SetBackgroundColor(theme::scheme().input),
        Print(" ".repeat((padding.0 + width + padding.1) as usize)),
        MoveTo(pos.0 + padding.0, pos.1),
        Print(buf),
        Print("▏"),
        ResetColor
    )
    .map_err(|_| EpError::Display)?;

    Ok(())
}

fn render_input_line(rel_i: u16) -> EpResult<()> {
    let name_col = 39;
    render_input((get_view_shift() + name_col, rel_i + 2), 20, (0, 5))?;

    Ok(())
}

fn render_file_line(
    rel_i: u16,
    is_cursor_pos: bool,
    file: &PathBuf,
    is_selected: bool,
) -> EpResult<()> {
    let c = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(theme::item_bg(is_selected, is_cursor_pos));
    let body_row = BodyRow::new(file, c.into(), under_name_color);
    di_view_line!(
        format!("{}{}", rel_i, body_row.gen_key()),
        rel_i + 2,
        Print(body_row),
    )
}

fn render_empty_line(rel_i: u16) -> EpResult<()> {
    if rel_i == 0 {
        let row = format!(
            "{}> | Press 'a' to create the New file | Empty",
            SetForegroundColor(theme::bar_color()),
        );
        di_view_line!(format!("{}", rel_i), rel_i + 2, Print(row))
    } else {
        di_view_line!(format!("{}", rel_i), rel_i + 2, Print(""))
    }
}

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
            let surround_color = SetBackgroundColor(theme::scheme().search_sur);
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

        format!("{}{:>8}", SetForegroundColor(theme::scheme().bsize), bod)
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
            SetForegroundColor(theme::scheme().mod_time),
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

fn render_menu() -> EpResult<()> {
    let slide_len = get_view_shift();
    if slide_len == 0 {
        return Ok(());
    }

    di_menu_line!(
        0,
        format!("{:?}", theme::scheme().label),
        Print(format!(
            "{} Select to Cd {}",
            SetBackgroundColor(theme::scheme().label),
            ResetColor
        ))
    )?;
    di_menu_line!(
        1,
        format!("{:?}", theme::wid_bar_color()),
        Print(colored_bar(theme::wid_bar_color(), slide_len - 1))
    )?;

    let menu = menu::refs();
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
    let tag = tag.chars().take(slide_len as usize - 6).collect::<String>();
    let cur = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(theme::widget_item_bg(is_cursor_pos, menu_enabled));

    di_menu_line!(
        row,
        format!("{}{}", cur, tag),
        Print(format!(
            "{} |{} {}{} {}{}",
            cur,
            under_name_color,
            SetForegroundColor(theme::scheme().menu_tag),
            tag,
            SetBackgroundColor(theme::widget_bg()),
            ResetColor,
        ))
    )?;
    Ok(())
}
