use crate::{
    canvas,
    config::{self, Config},
    global, handler, misc,
};
use crossterm::{cursor, terminal};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};
use tokio::time::{self, Duration, Instant};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to change the screen mode")]
    ScreenModeChangeFailed,

    #[error("filesystem error: {0}")]
    #[allow(clippy::enum_variant_names)]
    FilesystemError(String),

    #[error("The struct parsing failed: {0}")]
    TomlParseFailed(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Found error in running \"{0}\": {1}")]
    CommandExecutionFailed(String, String),

    #[error("Display the log failed")]
    LogDisplayFailed,

    #[error("The row rendering failed")]
    RowRenderingFailed,

    #[error("The input-area rendering failed")]
    InputRenderingFailed,

    #[error("Found platform error: {0}")]
    #[allow(clippy::enum_variant_names)]
    PlatformError(String),

    #[error("Screen flush failed: {0}")]
    ScreenFlushFailed(String),
}

impl Error {
    pub fn handle(self) {
        match self {
            Self::CommandExecutionFailed(cmd, kind) => {
                crate::log!("Failed to run \"{}\": {}", cmd, kind)
            }
            _ => panic!("{}", self),
        }
    }
}

#[macro_export]
macro_rules! global {
    (static $name:ident : $type:ty = $init:expr;) => {
        static $name: std::sync::LazyLock<$type> = std::sync::LazyLock::new(|| $init);
    };
}

#[macro_export]
macro_rules! sys_log {
    ($($fmt:expr),+) => {{
        let output_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
            .join(".local")
            .join("share")
            .join("endolphine")
            .join("log")
            .join(chrono::Local::now().format("%Y_%m_%d.log").to_string());
        let fmt_txt = format!("\n{}", format_args!($($fmt),+));

        use std::io::Write;

        let mut output_file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;
        output_file
            .write_all(fmt_txt.as_bytes())
            .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;

        Ok(())
    }}
}

pub fn enable_tui() -> Result<(), Error> {
    terminal::enable_raw_mode()
        .and_then(|_| {
            enable_render();
            crossterm::execute!(
                std::io::stdout(),
                terminal::EnterAlternateScreen,
                cursor::Hide,
                terminal::DisableLineWrap,
            )
        })
        .map_err(|_| Error::ScreenModeChangeFailed)
}

pub fn disable_tui() -> Result<(), Error> {
    terminal::disable_raw_mode()
        .and_then(|_| {
            disable_render();
            crossterm::execute!(
                std::io::stdout(),
                terminal::LeaveAlternateScreen,
                cursor::Show,
                terminal::EnableLineWrap,
            )
        })
        .map_err(|_| Error::ScreenModeChangeFailed)
}

pub async fn launch(path: &Path) -> Result<(), Error> {
    if !misc::exists_item(path) || path.is_file() {
        return Err(Error::InvalidArgument(format!(
            "invalid path (-> {})",
            path.to_string_lossy()
        )));
    }

    init(path)?;
    enable_tui()?;

    let quit_flag = Arc::new(AtomicBool::new(false));

    let event_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { event_handler(q) })
    };

    let ui_handle = {
        let q = quit_flag.clone();
        tokio::spawn(async move { ui(q).await })
    };

    event_handle.await.unwrap();
    ui_handle.await.unwrap();

    disable_tui()?;

    Ok(())
}

fn init(path: &Path) -> Result<(), Error> {
    let path = path
        .canonicalize()
        .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;

    set_path(&path);

    let c = misc::child_files_len(&path);
    crate::cursor::load().resize(c);

    if config::load().rm.for_tmp {
        let tmp_path = Path::new("/tmp").join("endolphine");
        if !tmp_path.exists() {
            std::fs::create_dir_all(tmp_path)
                .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;
        }
    }

    let log_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
        .join(".local")
        .join("share")
        .join("endolphine")
        .join("log");

    if !log_path.exists() {
        std::fs::create_dir_all(log_path)
            .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;
    }

    Ok(())
}

pub fn config_init() -> Result<(), Error> {
    let conf_path = config::file_path();
    if let Some(conf_path) = conf_path {
        if !conf_path.exists() {
            let parent = misc::parent(&conf_path);

            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;
            }

            let config_default = toml::to_string_pretty(&Config::default())
                .map_err(|e| Error::TomlParseFailed(e.to_string()))?;

            if !conf_path.exists() {
                std::fs::write(&conf_path, config_default)
                    .map_err(|e| Error::FilesystemError(e.kind().to_string()))?;
            }
        }
    }

    Ok(())
}

fn event_handler(quit_flag: Arc<AtomicBool>) {
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

async fn ui(quit_flag: Arc<AtomicBool>) {
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

global! {
    static PATH: RwLock<PathBuf> = RwLock::new(PathBuf::new());
}

pub fn get_path() -> PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: &Path) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path.to_path_buf();
}

global! {
    static RENDER: AtomicBool = AtomicBool::new(false);
}

pub fn disable_render() {
    RENDER.swap(false, Ordering::Relaxed);
}

pub fn enable_render() {
    RENDER.swap(true, Ordering::Relaxed);
}

global! {
    static GREP: RwLock<String> =  RwLock::new(String::new());
}

pub fn read_grep() -> String {
    let lock = GREP.read().unwrap();
    lock.to_owned()
}

pub fn grep_update<F: FnOnce(&mut String)>(f: F) {
    let mut lock = GREP.write().unwrap();
    f(&mut lock);
}

pub fn is_regex_empty() -> bool {
    GREP.read().unwrap().is_empty()
}

pub fn regex_match(buf: &str) -> bool {
    let lock = GREP.read().unwrap();

    let Ok(regex) = regex::Regex::new(&lock) else {
        return false;
    };

    regex.find(buf).is_some()
}

pub fn regex_range(buf: &str) -> Option<(usize, usize)> {
    let lock = GREP.read().unwrap();

    let Ok(regex) = regex::Regex::new(&lock) else {
        return None;
    };
    regex.find(buf).map(|m| (m.start(), m.end()))
}

pub fn sync_grep(input: &mut crate::input::Input) {
    crate::app::grep_update(|f| {
        *f = input
            .buffer_load()
            .clone()
            .and_then(|b| b.strip_prefix("/").map(|b| b.to_string()))
            .unwrap_or(" ".to_string())
    });
}

global! {
    static PROCS_COUNT: AtomicU16 = AtomicU16::new(0);
}

pub fn proc_count_up() {
    PROCS_COUNT.store(PROCS_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
}

pub fn proc_count_down() {
    PROCS_COUNT.store(PROCS_COUNT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
}

pub fn procs() -> u16 {
    PROCS_COUNT.load(Ordering::Relaxed)
}
