use crate::{
    canvas,
    config::{self, Config},
    error::*,
    global, handler, misc,
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};
use tokio::time::{self, Duration, Instant};

#[macro_export]
macro_rules! global {
    ($name:ident<$type:ty>, $init:expr, {
        $( $v:vis fn $fname:ident ( $($an:ident : $aty:ty),* ) $( -> $ret:ty )? $body:block )*
    }) => {
        static $name: std::sync::LazyLock<$type> = std::sync::LazyLock::new($init);

        $( $v fn $fname ($($an: $aty),*) $( -> $ret )? $body )*
    };
}

#[macro_export]
macro_rules! enable_tui {
    () => {
        'blk: {
            use crossterm::cursor;
            use crossterm::terminal;
            use std::io;
            if let Err(e) = terminal::enable_raw_mode() {
                break 'blk Err(e);
            }
            $crate::app::enable_render();
            crossterm::execute!(
                io::stdout(),
                terminal::EnterAlternateScreen,
                cursor::Hide,
                terminal::DisableLineWrap
            )
        }
        .map_err(|_| $crate::error::EpError::SwitchScreen)
    };
}

#[macro_export]
macro_rules! disable_tui {
    () => {
        'blk: {
            use crossterm::cursor;
            use crossterm::terminal;
            use std::io;
            if let Err(e) = terminal::disable_raw_mode() {
                break 'blk Err(e);
            }
            $crate::app::disable_render();
            crossterm::execute!(
                io::stdout(),
                terminal::LeaveAlternateScreen,
                cursor::Show,
                terminal::EnableLineWrap,
            )
        }
        .map_err(|_| $crate::error::EpError::SwitchScreen)
    };
}

global!(PATH<RwLock<PathBuf>>, || RwLock::new(PathBuf::new()), {
    pub fn get_path() -> PathBuf {
        (*PATH.read().unwrap()).clone()
    }

    pub fn set_path(new_path: &Path) {
        let mut lock = PATH.write().unwrap();
        *lock = new_path.to_path_buf();
    }
});

global!(RENDER<AtomicBool>, || AtomicBool::new(false), {
    pub fn disable_render() {
        RENDER.swap(false, Ordering::Relaxed);
    }

    pub fn enable_render() {
        RENDER.swap(true, Ordering::Relaxed);
    }
});

global!(GREP<RwLock<String>>, || RwLock::new(String::new()), {
    pub fn read_grep() -> String {
        let lock = GREP.read().unwrap();
        lock.to_owned()
    }
});

pub fn grep_update<F: FnOnce(&mut String)>(f: F) {
    let mut lock = GREP.write().unwrap();
    f(&mut lock);
}

pub fn is_match_grep<F: FnOnce(&str) -> bool>(f: F) -> bool {
    let lock = GREP.read().unwrap();
    f(&lock)
}

global!(PROCS_COUNT<AtomicU16>, || AtomicU16::new(0), {
    pub fn proc_count_up() {
        PROCS_COUNT.store(PROCS_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
    }

    pub fn proc_count_down() {
        PROCS_COUNT.store(PROCS_COUNT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
    }

    pub fn procs() -> u16 {
        PROCS_COUNT.load(Ordering::Relaxed)
    }
});

pub async fn launch(path: &Path) -> EpResult<()> {
    init(path)?;
    enable_tui!()?;

    let quit_flag = Arc::new(AtomicBool::new(false));

    let backend_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { backend(q) })
    };

    let ui_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { ui(q).await })
    };

    backend_handle.await.unwrap();
    ui_handle.await.unwrap();

    disable_tui!()?;

    Ok(())
}

fn init(path: &Path) -> EpResult<()> {
    let path = path
        .canonicalize()
        .map_err(|e| EpError::Init(e.kind().to_string()))?;

    set_path(&path);

    let c = misc::child_files_len(&path);
    crate::cursor::master().resize(c);

    if config::load().rm.for_tmp {
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

pub fn backend(quit_flag: Arc<AtomicBool>) {
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

        if RENDER.load(Ordering::Relaxed) {
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
