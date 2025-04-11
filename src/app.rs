use crate::{
    canvas,
    config::{self, Config},
    global, handler, misc,
};
use crossterm::{cursor, terminal};
use std::{
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicBool, AtomicU8, AtomicU16, Ordering},
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

    #[error("out log failed: {0}")]
    OutLogToFileFailed(String),
}

impl Error {
    pub fn handle(self) {
        match self {
            Self::CommandExecutionFailed(cmd, kind) => {
                crate::sys_log!("w", "Can't be execute \"{}\": {}", cmd, kind);
                crate::log!("Failed to run \"{}\": {}", cmd, kind);
            }
            Self::ScreenModeChangeFailed => {
                crate::sys_log!(
                    "e",
                    "Couldn't change the screen mode: disabled the mode in terminal or operation system"
                );

                panic!("{}", self);
            }
            Self::LogDisplayFailed | Self::RowRenderingFailed | Self::InputRenderingFailed => {
                crate::sys_log!("e", "Rendering failed");

                panic!("{}", self);
            }
            Self::ScreenFlushFailed(_) => {
                crate::sys_log!("e", "The stdout can't flush");

                panic!("{}", self);
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
    ($lv:expr, $($fmt:expr),+) => {{
        let now = chrono::Local::now();
        let output_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
            .join(".local")
            .join("share")
            .join("endolphine")
            .join("log")
            .join(now.format("%Y_%m_%d.log").to_string());
        let log_header = now.format("[%H:%M:%S]").to_string();
        let log_lvl = match $lv.to_ascii_lowercase().as_str() {
            "warn" | "w" => "[WARN]",
            "error" | "err" | "e" => "[ERROR]",
            "info" | "i" => "[INFO]",
            _ => "[INFO]",
        };
        let fmt_txt = format!("\n{} {} {}", log_header, log_lvl, format_args!($($fmt),+));

        use std::io::Write;

        let mut output_file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .map_err(|e| $crate::app::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
        output_file
            .write_all(fmt_txt.as_bytes())
            .map_err(|e| $crate::app::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
    }};
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
        sys_log!("e", "Invalid input detected");

        return Err(Error::InvalidArgument(format!(
            "invalid path (-> {})",
            path.to_string_lossy()
        )));
    }

    init(path)?;
    enable_tui()?;

    sys_log!(
        "i",
        "Endolphine launch in {} successfully",
        path.canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path.to_string_lossy().to_string())
    );

    if config::check().is_err() {
        crate::log!("Failed load config.toml, use the Default config");
    }

    let event_handle = tokio::spawn(async move { event_handler() });

    let ui_handle = tokio::spawn(async move { ui().await });

    event_handle.await.unwrap();
    ui_handle.await.unwrap();

    disable_tui()?;

    Ok(())
}

fn init(path: &Path) -> Result<(), Error> {
    let path = path.canonicalize().map_err(|e| {
        crate::sys_log!("e", "Couldn't get the canonicalized path");
        Error::FilesystemError(e.kind().to_string())
    })?;

    set_path(&path);

    let c = misc::child_files_len(&path);
    crate::cursor::load().resize(c);

    if config::load().delete.for_tmp {
        let tmp_path = Path::new("/tmp").join("endolphine");
        if !tmp_path.exists() {
            std::fs::create_dir_all(tmp_path).map_err(|e| {
                crate::sys_log!("e", "Couldn't create the \"/tmp/\"");
                Error::FilesystemError(e.kind().to_string())
            })?;
        }
    }

    init_keymapping();

    let log_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
        .join(".local")
        .join("share")
        .join("endolphine")
        .join("log");

    if !log_path.exists() {
        std::fs::create_dir_all(log_path).map_err(|e| {
            crate::sys_log!("e", "Couldn't create the log directory");
            Error::FilesystemError(e.kind().to_string())
        })?;
    }

    Ok(())
}

pub fn config_init() -> Result<(), Error> {
    let conf_path = config::file_path();
    if let Some(conf_path) = conf_path {
        if !conf_path.exists() {
            let parent = misc::parent(&conf_path);

            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration dir");
                    Error::FilesystemError(e.kind().to_string())
                })?;
            }

            let config_default = toml::to_string_pretty(&Config::default()).map_err(|e| {
                crate::sys_log!("e", "Couldn't generate the default configration");
                Error::TomlParseFailed(e.to_string())
            })?;

            if !conf_path.exists() {
                std::fs::write(&conf_path, config_default).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration file");
                    Error::FilesystemError(e.kind().to_string())
                })?;
            }
        }
    }

    Ok(())
}

fn init_keymapping() {
    use crate::command;
    use crate::key::Keymap;
    use AppMode::{Normal, Visual};
    use config::register_key;

    register_key(Normal, "ZZ".into(), command::ExitApp);
    register_key(Normal, "<ESC>".into(), command::ResetView);
    register_key(Normal, "k".into(), command::MoveUp);
    register_key(Normal, "K".into(), command::Remapping(Keymap::from("10k")));
    register_key(Normal, "j".into(), command::MoveDown);
    register_key(Normal, "J".into(), command::Remapping(Keymap::from("10j")));
    register_key(Normal, "h".into(), command::MoveParent);
    register_key(Normal, "l".into(), command::EnterDirOrEdit);
    register_key(Normal, "V".into(), command::VisualSelect);
    register_key(Normal, "M".into(), command::MenuToggle);
    register_key(Normal, "m".into(), command::MenuMove);
    register_key(Normal, "a".into(), command::AskCreate);
    if config::load().delete.ask {
        register_key(Normal, "d".into(), command::AskDelete);
    } else {
        register_key(
            Normal,
            "dd".into(),
            command::DeleteFileOrDir {
                use_tmp: config::load().delete.for_tmp,
                yank_and_native: (config::load().delete.yank, config::load().native_clip),
            },
        );
    }
    register_key(Normal, "r".into(), command::AskRename);
    register_key(
        Normal,
        "yy".into(),
        command::Yank {
            native: config::load().native_clip,
        },
    );
    register_key(Normal, "p".into(), command::AskPaste);
    register_key(Normal, "/".into(), command::Search);
    register_key(Normal, "n".into(), command::SearchNext);

    register_key(Visual, "ZZ".into(), command::ExitApp);
    register_key(Visual, "<ESC>".into(), command::ResetView);
    register_key(Visual, "k".into(), command::MoveUp);
    register_key(Visual, "K".into(), command::Remapping(Keymap::from("10k")));
    register_key(Visual, "j".into(), command::MoveDown);
    register_key(Visual, "J".into(), command::Remapping(Keymap::from("10j")));
    register_key(Visual, "h".into(), command::MoveParent);
    register_key(Visual, "l".into(), command::EnterDirOrEdit);
    register_key(Visual, "V".into(), command::VisualSelect);
    register_key(Visual, "M".into(), command::MenuToggle);
    register_key(Visual, "m".into(), command::MenuMove);
    register_key(Visual, "a".into(), command::AskCreate);
    if config::load().delete.ask {
        register_key(Visual, "d".into(), command::AskDelete);
    } else {
        register_key(
            Visual,
            "d".into(),
            command::DeleteSelected {
                use_tmp: config::load().delete.for_tmp,
                yank_and_native: (config::load().delete.yank, config::load().native_clip),
            },
        );
    }
    register_key(Visual, "r".into(), command::AskRename);
    register_key(
        Visual,
        "y".into(),
        command::Yank {
            native: config::load().native_clip,
        },
    );
    register_key(Visual, "p".into(), command::AskPaste);
    register_key(Visual, "/".into(), command::Search);
    register_key(Visual, "n".into(), command::SearchNext);
}

fn event_handler() {
    loop {
        if let Err(e) = handler::handle_event() {
            e.handle();
        }
    }
}

async fn ui() {
    loop {
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
    static MODE: AtomicU8 = AtomicU8::new(AppMode::Normal as u8);
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AppMode {
    Normal = 0b0001,
    Visual = 0b0010,
    // TODO
    // Command = 0b0100,
}

pub fn current_mode() -> AppMode {
    let loaded = MODE.load(Ordering::Relaxed);

    if loaded != AppMode::Normal as u8 && loaded != AppMode::Visual as u8
    /* && loaded != AppMode::Command as u8 */
    {
        crate::sys_log!("e", "Unknown app mode: {}", loaded);
        panic!("unknown mode");
    }

    unsafe { std::mem::transmute(loaded) }
}

pub fn switch_mode(mode: AppMode) {
    MODE.swap(mode as u8, Ordering::Relaxed);
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

pub fn regex_match(buf: &str) -> bool {
    let lock = GREP.read().unwrap();

    let Ok(regex) = regex::Regex::new(&lock) else {
        return false;
    };

    regex.find(buf).is_some()
}

pub fn regex_range(buf: &str) -> Option<(usize, usize)> {
    let lock = GREP.read().unwrap();

    if lock.is_empty() {
        return None;
    }

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

global! {
    static KEYBUF: RwLock<Vec<crate::key::Key>> = RwLock::new(vec![]);
}

pub fn push_key_buf(key: crate::key::Key) {
    KEYBUF.write().unwrap().push(key);
}

pub fn sync_key_buf(other: crate::key::Keymap) {
    *KEYBUF.write().unwrap() = other.into();
}

pub fn clear_key_buf() {
    KEYBUF.write().unwrap().clear();
}

pub fn load_buf() -> Vec<crate::key::Key> {
    KEYBUF.read().unwrap().clone()
}
