use crate::{app, global, theme};
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
    ($($args:expr),+) => {{
        use crossterm::cursor;
        use crossterm::style;
        use crossterm::terminal;
        use crossterm::terminal::ClearType;
        use std::io;
        let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
        crossterm::queue!(
            io::stdout(),
            style::ResetColor,
            cursor::MoveTo(0, row),
            style::Print(format_args!($($args),+)),
            terminal::Clear(ClearType::UntilNewLine)
        )
        .unwrap_or_else(|_| {
            $crate::app::Error::LogDisplayFailed.handle();
        });
    }};

}

#[macro_export]
macro_rules! dbg_log {
    ($($args:expr),+, $is_dbg:expr) => {{
        use crossterm::cursor;
        use crossterm::style;
        use crossterm::terminal;
        use crossterm::terminal::ClearType;
        use std::io;
        let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
        let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
        if let Err(_) = crossterm::execute!(
            io::stdout(),
            cursor::MoveTo(0, row),
            style::Print(format!("{} {}", ts, format_args!($($args),+))),
            terminal::Clear(ClearType::UntilNewLine),
        ) {
            $crate::error::EpError::Display.handle()
        };
    }};
}

trait Widget {
    const ID: u8;

    fn cached_render_row<D: std::fmt::Display>(
        tag: &str,
        row: u16,
        cmds: D,
    ) -> Result<(), app::Error> {
        if !cache_match((row, Self::ID), tag) {
            cache_insert((row, Self::ID), tag.to_string());
            Self::render_row(row, cmds.to_string()).map_err(|_| {
                crate::sys_log!("e", "The widget rendering failed: ID={}", Self::ID);
                app::Error::RowRenderingFailed
            })
        } else {
            Ok(())
        }
    }

    fn render(size: (u16, u16)) -> Result<(), app::Error>;

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

pub fn render() -> Result<(), app::Error> {
    let (width, height) = crossterm::terminal::size().map_err(|e| {
        crate::sys_log!("e", "Couldn't get the terminal size");
        app::Error::PlatformError(e.kind().to_string())
    })?;

    if height <= 4 {
        return Ok(());
    }

    header::Header::render((width, height))?;

    if height > 4 {
        body::Body::render((width, height))?;
    }

    footer::Footer::render((width, height))?;

    if width > 0 {
        menu::Menu::render((width, height))?;
    }

    use std::io::Write;

    std::io::stdout()
        .flush()
        .map_err(|e| app::Error::ScreenFlushFailed(e.kind().to_string()))?;

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

global! {
    static VIEW_SHIFT: AtomicU16 = AtomicU16::new(0);
}

pub fn get_view_shift() -> u16 {
    VIEW_SHIFT.load(Ordering::Relaxed)
}

pub fn set_view_shift(new_value: u16) {
    VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
}

global! {
    static CACHE: RwLock<HashMap<(u16, u8), String>> = RwLock::new(HashMap::new());
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
