use crate::{disable_tui, enable_tui, error::*, thread};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use once_cell::sync::OnceCell;
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, RwLock},
};

const PATH: OnceCell<RwLock<PathBuf>> = OnceCell::new();

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
    if let Err(_) =
        PATH.get_or_try_init(|| -> Result<RwLock<PathBuf>, ()> { Ok(RwLock::new(path.clone())) })
    {
        return Err(EpError::InitFailed);
    }

    Ok(())
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
