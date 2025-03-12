use crate::{error::*, global, theme};
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::{
    collections::HashMap,
    sync::{
        RwLock,
        atomic::{AtomicU16, Ordering},
    },
};

mod body;
mod footer;
mod header;
mod menu;

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

global! {
    const VIEW_SHIFT: AtomicU16 = AtomicU16::new(0);
}

pub fn get_view_shift() -> u16 {
    VIEW_SHIFT.load(Ordering::Relaxed)
}

pub fn set_view_shift(new_value: u16) {
    VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
}

global! {
    const CACHE: RwLock<HashMap<(u16, u8), String>> = RwLock::new(HashMap::new());
}

pub fn cache_insert(key: (u16, u8), tag: String) {
    CACHE.write().unwrap().insert(key, tag);
}

pub fn cache_match(key: (u16, u8), tag: &str) -> bool {
    CACHE.read().unwrap().get(&key).map(|c| c.as_ref()) == Some(tag)
}

pub fn cache_clear() {
    CACHE.write().unwrap().clear();
}

trait Widget {
    const ID: u8;

    fn cached_render_row(tag: &str, row: u16, cmds: String) -> EpResult<()> {
        if !cache_match((row, Self::ID), tag) {
            cache_insert((row, Self::ID), tag.to_string());
            Self::render_row(row, cmds).map_err(|_| EpError::Display)
        } else {
            Ok(())
        }
    }

    fn render(size: (u16, u16)) -> EpResult<()>;

    fn render_row(row: u16, cmds: String) -> std::io::Result<()> {
        crossterm::queue!(
            std::io::stdout(),
            MoveTo(get_view_shift(), row),
            SetForegroundColor(theme::app_fg()),
            SetBackgroundColor(theme::app_bg()),
            Clear(ClearType::UntilNewLine),
            Print(cmds),
            ResetColor
        )
    }
}

pub fn render() -> EpResult<()> {
    let (width, height) = crossterm::terminal::size().unwrap_or((0, 0));
    let size = crossterm::terminal::size().unwrap_or((0, 0));

    if height <= 4 {
        return Ok(());
    }

    header::Header::render(size)?;

    if height > 4 {
        body::Body::render(size)?;
    }

    footer::Footer::render(size)?;

    if width > 0 {
        menu::Menu::render(size)?;
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
