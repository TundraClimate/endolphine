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
            .map_err(|e| $crate::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
        output_file
            .write_all(fmt_txt.as_bytes())
            .map_err(|e| $crate::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
    }};
}

pub fn enable_tui() -> Result<(), crate::Error> {
    crossterm::terminal::enable_raw_mode()
        .and_then(|_| {
            enable_render();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::cursor::Hide,
                crossterm::terminal::DisableLineWrap,
            )
        })
        .map_err(|_| crate::Error::ScreenModeChangeFailed)
}

pub fn disable_tui() -> Result<(), crate::Error> {
    crossterm::terminal::disable_raw_mode()
        .and_then(|_| {
            disable_render();
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::terminal::EnableLineWrap,
            )
        })
        .map_err(|_| crate::Error::ScreenModeChangeFailed)
}

pub async fn launch(path: &std::path::Path) -> Result<(), crate::Error> {
    if !crate::misc::exists_item(path) || path.is_file() {
        sys_log!("e", "Invalid input detected");

        return Err(crate::Error::InvalidArgument(format!(
            "invalid path (-> {})",
            path.to_string_lossy()
        )));
    }

    crate::initialize::application(path)?;
    crate::initialize::keymap()?;
    enable_tui()?;

    sys_log!(
        "i",
        "Endolphine launch in {} successfully",
        path.canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path.to_string_lossy().to_string())
    );

    if crate::config::check().is_err() {
        crate::log!("Failed load config.toml, use the Default config");
    }

    let event_handle = tokio::spawn(async move { event_handler() });

    let ui_handle = tokio::spawn(async move { ui().await });

    event_handle.await.unwrap();
    ui_handle.await.unwrap();

    disable_tui()?;

    Ok(())
}

fn event_handler() {
    loop {
        if let Err(e) = handle_event() {
            e.handle();
        }
    }
}

pub fn handle_event() -> Result<(), crate::Error> {
    match crossterm::event::read() {
        Ok(crossterm::event::Event::Key(key)) => {
            {
                let key = crate::key::Key::from_keyevent(&key);

                push_key_buf(key);
            }

            if matches!(current_mode()?, AppMode::Input) {
                if let Some(cmd_res) = crate::config::eval_input_keymap(&load_buf()) {
                    clear_key_buf();
                    cmd_res?
                }
            }

            if !crate::config::has_similar_map(&load_buf(), current_mode()?) {
                clear_key_buf();

                return Ok(());
            }

            if let Some(cmd_res) = crate::config::eval_keymap(current_mode()?, &load_buf()) {
                clear_key_buf();
                cmd_res?
            }
        }
        Ok(crossterm::event::Event::Resize(_, _)) => {
            crate::cursor::load().resize(crate::misc::child_files_len(&get_path()));
            crate::canvas::cache_clear();
        }
        _ => {}
    }

    Ok(())
}

async fn ui() {
    loop {
        let start = tokio::time::Instant::now();

        if RENDER.load(std::sync::atomic::Ordering::Relaxed) {
            if let Err(e) = crate::canvas::render() {
                e.handle();
            }
        }

        let elapsed = start.elapsed();
        let tick = 70;
        if elapsed < tokio::time::Duration::from_millis(tick) {
            tokio::time::sleep(tokio::time::Duration::from_millis(tick) - elapsed).await;
        }
    }
}

global! {
    static PATH: std::sync::RwLock<std::path::PathBuf> = std::sync::RwLock::new(std::path::PathBuf::new());
}

pub fn get_path() -> std::path::PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: &std::path::Path) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path.to_path_buf();
}

global! {
    static RENDER: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
}

pub fn disable_render() {
    RENDER.swap(false, std::sync::atomic::Ordering::Relaxed);
}

pub fn enable_render() {
    RENDER.swap(true, std::sync::atomic::Ordering::Relaxed);
}

global! {
    static MODE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(AppMode::Normal as u8);
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum AppMode {
    Normal,
    Visual,
    Input,
    // TODO
    // Command,
}

pub fn current_mode() -> Result<AppMode, crate::Error> {
    let loaded = MODE.load(std::sync::atomic::Ordering::Relaxed);

    if loaded != AppMode::Normal as u8
        && loaded != AppMode::Visual as u8
        && loaded != AppMode::Input as u8
    /* && loaded != AppMode::Command as u8 */
    {
        crate::sys_log!("e", "Unknown app mode: {}", loaded);
        return Err(crate::Error::IncorrectProgram(
            String::from("app::current_mode"),
            String::from("Loaded invalid mode"),
        ));
    }

    let converted = match loaded {
        0 => AppMode::Normal,
        1 => AppMode::Visual,
        2 => AppMode::Input,
        _ => unreachable!(),
    };

    Ok(converted)
}

pub fn switch_mode(mode: AppMode) {
    MODE.swap(mode as u8, std::sync::atomic::Ordering::Relaxed);
}

global! {
    static GREP: std::sync::RwLock<String> =  std::sync::RwLock::new(String::new());
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

pub fn sync_grep(input: &str) {
    crate::app::grep_update(|f| {
        *f = input
            .strip_prefix("/")
            .map(|b| b.to_string())
            .unwrap_or(" ".to_string())
    });
}

global! {
    static PROCS_COUNT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);
}

pub fn proc_count_up() {
    PROCS_COUNT.store(
        PROCS_COUNT.load(std::sync::atomic::Ordering::Relaxed) + 1,
        std::sync::atomic::Ordering::Relaxed,
    );
}

pub fn proc_count_down() {
    PROCS_COUNT.store(
        PROCS_COUNT.load(std::sync::atomic::Ordering::Relaxed) - 1,
        std::sync::atomic::Ordering::Relaxed,
    );
}

pub fn procs() -> u16 {
    PROCS_COUNT.load(std::sync::atomic::Ordering::Relaxed)
}

global! {
    static KEYBUF: std::sync::RwLock<Vec<crate::key::Key>> = std::sync::RwLock::new(vec![]);
}

pub fn push_key_buf(key: crate::key::Key) {
    KEYBUF.write().unwrap().push(key);
}

pub fn sync_key_buf(other: crate::key::Keymap) {
    *KEYBUF.write().unwrap() = other.as_vec().to_owned();
}

pub fn clear_key_buf() {
    KEYBUF.write().unwrap().clear();
}

pub fn load_buf() -> Vec<crate::key::Key> {
    KEYBUF.read().unwrap().clone()
}
