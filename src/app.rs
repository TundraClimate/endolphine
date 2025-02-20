use crate::{
    canvas,
    config::{self, Config},
    error::*,
    global, handler, misc,
};
use std::{
    path::Path,
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
            $crate::global::enable_render();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::cursor::Hide,
                crossterm::terminal::DisableLineWrap
            )
        }
        .map_err(|_| $crate::error::EpError::SwitchScreen)
    };
}

#[macro_export]
macro_rules! disable_tui {
    () => {
        'blk: {
            if let Err(e) = crossterm::terminal::disable_raw_mode() {
                break 'blk Err(e);
            }
            $crate::global::disable_render();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::terminal::EnableLineWrap,
            )
        }
        .map_err(|_| $crate::error::EpError::SwitchScreen)
    };
}

pub async fn launch(path: &Path) -> EpResult<()> {
    init(path)?;
    enable_tui!()?;

    let quit_flag = Arc::new(AtomicBool::new(false));

    let process_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { process(q) })
    };

    let ui_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { ui(q).await })
    };

    process_handle.await.unwrap();
    ui_handle.await.unwrap();

    disable_tui!()?;

    Ok(())
}

fn init(path: &Path) -> EpResult<()> {
    let path = path
        .canonicalize()
        .map_err(|e| EpError::Init(e.kind().to_string()))?;

    global::init(&path)?;

    if global::config().rm.for_tmp {
        let tmp_path = Path::new("/tmp").join("endolphine");
        if !tmp_path.exists() {
            std::fs::create_dir_all(tmp_path).map_err(|e| EpError::Init(e.to_string()))?;
        }
    }

    Ok(())
}

pub fn config_init() -> EpResult<()> {
    let conf_path = config::file_path();
    if let Some(conf_path) = conf_path {
        if !conf_path.exists() {
            let parent = misc::parent(&conf_path);

            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| EpError::Init(e.kind().to_string()))?;
            }

            let config_default = toml::to_string_pretty(&Config::default())
                .map_err(|e| EpError::Init(e.to_string()))?;

            if !conf_path.exists() {
                std::fs::write(&conf_path, config_default)
                    .map_err(|e| EpError::Init(e.kind().to_string()))?;
            }
        }
    }

    Ok(())
}

pub fn process(quit_flag: Arc<AtomicBool>) {
    loop {
        match handler::handle_event() {
            Ok(is_quit) => {
                if is_quit {
                    quit_flag.swap(true, Ordering::Relaxed);
                    break;
                }
            }
            Err(e) => e.handle(),
        }
    }
}

pub async fn ui(quit_flag: Arc<AtomicBool>) {
    while !quit_flag.load(Ordering::Relaxed) {
        let start = Instant::now();

        if global::is_render() {
            if let Err(e) = canvas::render() {
                e.handle();
            }
        }

        let elapsed = start.elapsed();
        let tick = 70;
        if elapsed < Duration::from_millis(tick) {
            time::sleep(Duration::from_millis(tick) - elapsed).await;
        }
    }
}
