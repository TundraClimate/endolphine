use crate::{canvas, error::*, event_handler, global};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::time::{self, Duration, Instant};

#[macro_export]
macro_rules! enable_tui {
    () => {
        'blk: {
            if let Err(e) = crossterm::terminal::enable_raw_mode() {
                break 'blk Err(e);
            }
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::cursor::Hide,
                crossterm::terminal::DisableLineWrap
            )
        }
        .map_err(|_| crate::error::EpError::SwitchScreen)
    };
}

#[macro_export]
macro_rules! disable_tui {
    () => {
        'blk: {
            if let Err(e) = crossterm::terminal::disable_raw_mode() {
                break 'blk Err(e);
            }
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::terminal::EnableLineWrap,
            )
        }
        .map_err(|_| crate::error::EpError::SwitchScreen)
    };
}

pub async fn launch(path: &PathBuf) -> EpResult<()> {
    init(path)?;
    enable_tui!()?;

    let quit_flag = Arc::new(AtomicBool::new(false));

    let process_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { process(q).await })
    };

    let ui_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { ui(q).await })
    };

    process_handle.await.unwrap()?;
    ui_handle.await.unwrap()?;

    disable_tui!()?;

    Ok(())
}

fn init(path: &PathBuf) -> EpResult<()> {
    let Ok(path) = path.canonicalize() else {
        return Err(EpError::InitFailed);
    };

    global::init(&path)?;

    Ok(())
}

pub async fn process(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    loop {
        if event_handler::handle_event().await? {
            quit_flag.swap(true, Ordering::Relaxed);
            break;
        }
    }

    Ok(())
}

pub async fn ui(quit_flag: Arc<AtomicBool>) -> EpResult<()> {
    while !quit_flag.load(Ordering::Relaxed) {
        let start = Instant::now();

        {
            canvas::render()?;
        }

        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(50) {
            time::sleep(Duration::from_millis(50) - elapsed).await;
        }
    }

    Ok(())
}
