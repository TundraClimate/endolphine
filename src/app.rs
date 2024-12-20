use crate::{disable_tui, enable_tui, error::*, thread};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
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

    Ok(())
}

pub fn get_path() -> PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: PathBuf) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path;
}

pub fn get_row() -> u16 {
    ROW.load(Ordering::Relaxed)
}

pub fn set_row(new_value: u16) {
    ROW.swap(new_value, Ordering::Relaxed);
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
            enable_raw_mode()?;
            execute!(
                std::io::stdout(),
                EnterAlternateScreen,
                Hide,
                DisableLineWrap
            )
        })()
    };
}

#[macro_export]
macro_rules! disable_tui {
    () => {
        (|| -> std::io::Result<()> {
            disable_raw_mode()?;
            execute!(
                std::io::stdout(),
                LeaveAlternateScreen,
                Show,
                EnableLineWrap,
            )
        })()
    };
}
