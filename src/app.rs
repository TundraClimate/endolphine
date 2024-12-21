use crate::{cursor::Cursor, disable_tui, enable_tui, error::*, misc, thread};
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering},
        Arc, RwLock,
    },
};

static PATH: Lazy<RwLock<PathBuf>> = Lazy::new(|| RwLock::new(PathBuf::new()));

static ROW: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(100));

static CURSOR: Lazy<Cursor> = Lazy::new(|| Cursor::new());

static VIEW_SHIFT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(0));

pub async fn launch(path: &PathBuf) -> EpResult<()> {
    init(path)?;
    enable_tui!().map_err(|_| EpError::SwitchScreen)?;

    let quit_flag = Arc::new(AtomicBool::new(false));

    let process_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { thread::process(q).await })
    };

    let ui_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { thread::ui(q).await })
    };

    process_handle.await.unwrap()?;
    ui_handle.await.unwrap()?;

    disable_tui!().map_err(|_| EpError::SwitchScreen)?;

    Ok(())
}

fn init(path: &PathBuf) -> EpResult<()> {
    let Ok(path) = path.canonicalize() else {
        return Err(EpError::InitFailed);
    };

    let mut lock = PATH.write().unwrap();
    *lock = path.clone();

    let (_, row) = crossterm::terminal::size().map_err(|_| EpError::InitFailed)?;
    ROW.swap(row, Ordering::Relaxed);

    let c = misc::child_files(&path).len();
    CURSOR.resize(c);

    Ok(())
}

pub fn get_path() -> PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: &PathBuf) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path.clone();
}

pub fn get_row() -> u16 {
    ROW.load(Ordering::Relaxed)
}

pub fn set_row(new_value: u16) {
    ROW.swap(new_value, Ordering::Relaxed);
}

pub fn cursor() -> &'static Cursor {
    &*CURSOR
}

pub fn get_view_shift() -> u16 {
    VIEW_SHIFT.load(Ordering::Relaxed)
}

pub fn set_view_shift(new_value: u16) {
    VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
}

#[macro_export]
macro_rules! enable_tui {
    () => {
        (|| -> std::io::Result<()> {
            crossterm::terminal::enable_raw_mode()?;
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::cursor::Hide,
                crossterm::terminal::DisableLineWrap
            )
        })()
    };
}

#[macro_export]
macro_rules! disable_tui {
    () => {
        (|| -> std::io::Result<()> {
            crossterm::terminal::disable_raw_mode()?;
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::terminal::EnableLineWrap,
            )
        })()
    };
}
