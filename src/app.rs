use crate::{disable_tui, enable_tui, error::*, thread};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use once_cell::sync::{Lazy, OnceCell};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU16, Ordering},
        Arc,
    },
};

static PATH: OnceCell<PathBuf> = OnceCell::new();

static ROW: OnceCell<u16> = OnceCell::new();

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

    if let Err(_) = PATH.get_or_try_init(|| -> Result<PathBuf, ()> { Ok(path.clone()) }) {
        return Err(EpError::InitFailed);
    }

    let (_, row) = crossterm::terminal::size().map_err(|_| EpError::InitFailed)?;
    if let Err(_) = ROW.get_or_try_init(|| -> Result<u16, ()> { Ok(row) }) {
        return Err(EpError::InitFailed);
    }

    Ok(())
}

pub fn get_path() -> EpResult<PathBuf> {
    Ok(PATH.get().ok_or(EpError::InitFailed)?.clone())
}

pub fn set_path(new_path: PathBuf) {
    PATH.set(new_path).unwrap();
}

pub fn get_row() -> EpResult<u16> {
    Ok(*ROW.get().ok_or(EpError::InitFailed)?)
}

pub fn set_row(new_value: u16) {
    ROW.set(new_value).unwrap();
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
