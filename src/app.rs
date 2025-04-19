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

    init(path)?;
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

fn init(path: &std::path::Path) -> Result<(), crate::Error> {
    let path = path.canonicalize().map_err(|e| {
        crate::sys_log!("e", "Couldn't get the canonicalized path");
        crate::Error::FilesystemError(e.kind().to_string())
    })?;

    set_path(&path);

    let c = crate::misc::child_files_len(&path);
    crate::cursor::load().resize(c);

    if crate::config::load().delete.for_tmp {
        let tmp_path = std::path::Path::new("/tmp").join("endolphine");
        if !tmp_path.exists() {
            std::fs::create_dir_all(tmp_path).map_err(|e| {
                crate::sys_log!("e", "Couldn't create the \"/tmp/\"");
                crate::Error::FilesystemError(e.kind().to_string())
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
            crate::Error::FilesystemError(e.kind().to_string())
        })?;
    }

    Ok(())
}

pub fn config_init() -> Result<(), crate::Error> {
    let conf_path = crate::config::file_path();
    if let Some(conf_path) = conf_path {
        if !conf_path.exists() {
            let parent = crate::misc::parent(&conf_path);

            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration dir");
                    crate::Error::FilesystemError(e.kind().to_string())
                })?;
            }

            let config_default = toml::to_string_pretty(&crate::config::Config::default())
                .map_err(|e| {
                    crate::sys_log!("e", "Couldn't generate the default configration");
                    crate::Error::TomlParseFailed(e.to_string())
                })?;

            if !conf_path.exists() {
                std::fs::write(&conf_path, config_default).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration file");
                    crate::Error::FilesystemError(e.kind().to_string())
                })?;
            }
        }
    }

    Ok(())
}

fn init_keymapping() {
    use crate::command;
    use crate::config::register_key;
    use AppMode::{Input, Normal, Visual};

    register_key(Normal, "ZZ".into(), command::ExitApp);
    register_key(Normal, "<ESC>".into(), command::ResetView);
    register_key(Normal, "k".into(), command::MoveUp);
    register_key(Normal, "j".into(), command::MoveDown);
    register_key(Normal, "gg".into(), command::MoveTop);
    register_key(Normal, "G".into(), command::MoveBottom);
    register_key(Normal, "gk".into(), command::PageUp);
    register_key(Normal, "gj".into(), command::PageDown);
    register_key(Normal, "h".into(), command::MoveParent);
    register_key(Normal, "l".into(), command::EnterDirOrEdit);
    register_key(Normal, "V".into(), command::VisualSelect);
    register_key(Normal, "M".into(), command::MenuToggle);
    register_key(Normal, "m".into(), command::MenuMove);
    register_key(Normal, "a".into(), command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Normal, "d".into(), command::AskDelete);
    } else {
        register_key(
            Normal,
            "dd".into(),
            command::DeleteFileOrDir {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Normal, "r".into(), command::AskRename);
    register_key(
        Normal,
        "yy".into(),
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Normal, "p".into(), command::AskPaste);
    register_key(Normal, "/".into(), command::Search);
    register_key(Normal, "n".into(), command::SearchNext);

    register_key(Visual, "ZZ".into(), command::ExitApp);
    register_key(Visual, "<ESC>".into(), command::ResetView);
    register_key(Visual, "k".into(), command::MoveUp);
    register_key(Visual, "j".into(), command::MoveDown);
    register_key(Visual, "gg".into(), command::MoveTop);
    register_key(Visual, "G".into(), command::MoveBottom);
    register_key(Visual, "gk".into(), command::PageUp);
    register_key(Visual, "gj".into(), command::PageDown);
    register_key(Visual, "h".into(), command::MoveParent);
    register_key(Visual, "l".into(), command::EnterDirOrEdit);
    register_key(Visual, "V".into(), command::VisualSelect);
    register_key(Visual, "M".into(), command::MenuToggle);
    register_key(Visual, "m".into(), command::MenuMove);
    register_key(Visual, "a".into(), command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Visual, "d".into(), command::AskDelete);
    } else {
        register_key(
            Visual,
            "d".into(),
            command::DeleteSelected {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Visual, "r".into(), command::AskRename);
    register_key(
        Visual,
        "y".into(),
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Visual, "p".into(), command::AskPaste);
    register_key(Visual, "/".into(), command::Search);
    register_key(Visual, "n".into(), command::SearchNext);

    register_key(Input, "<ESC>".into(), command::DisableInput);
    register_key(Input, "<CR>".into(), command::CompleteInput);
    register_key(Input, "<c-h>".into(), command::InputCursorPrev);
    register_key(Input, "<c-l>".into(), command::InputCursorNext);
    register_key(Input, "<BS>".into(), command::InputDeleteCurrent);
    register_key(Input, "<s-BS>".into(), command::InputDeleteNext);
    register_key(Input, "<SPACE>".into(), command::InputInsert(' '));
    register_key(Input, "!".into(), command::InputInsert('!'));
    register_key(Input, "\"".into(), command::InputInsert('"'));
    register_key(Input, "#".into(), command::InputInsert('#'));
    register_key(Input, "$".into(), command::InputInsert('$'));
    register_key(Input, "%".into(), command::InputInsert('%'));
    register_key(Input, "&".into(), command::InputInsert('&'));
    register_key(Input, "'".into(), command::InputInsert('\''));
    register_key(Input, "(".into(), command::InputInsert('('));
    register_key(Input, ")".into(), command::InputInsert(')'));
    register_key(Input, "*".into(), command::InputInsert('*'));
    register_key(Input, "+".into(), command::InputInsert('+'));
    register_key(Input, ",".into(), command::InputInsert(','));
    register_key(Input, "-".into(), command::InputInsert('-'));
    register_key(Input, ".".into(), command::InputInsert('.'));
    register_key(Input, "/".into(), command::InputInsert('/'));
    register_key(Input, "0".into(), command::InputInsert('0'));
    register_key(Input, "1".into(), command::InputInsert('1'));
    register_key(Input, "2".into(), command::InputInsert('2'));
    register_key(Input, "3".into(), command::InputInsert('3'));
    register_key(Input, "4".into(), command::InputInsert('4'));
    register_key(Input, "5".into(), command::InputInsert('5'));
    register_key(Input, "6".into(), command::InputInsert('6'));
    register_key(Input, "7".into(), command::InputInsert('7'));
    register_key(Input, "8".into(), command::InputInsert('8'));
    register_key(Input, "9".into(), command::InputInsert('9'));
    register_key(Input, ":".into(), command::InputInsert(':'));
    register_key(Input, ";".into(), command::InputInsert(';'));
    register_key(Input, "<lt>".into(), command::InputInsert('<'));
    register_key(Input, "=".into(), command::InputInsert('='));
    register_key(Input, ">".into(), command::InputInsert('>'));
    register_key(Input, "?".into(), command::InputInsert('?'));
    register_key(Input, "@".into(), command::InputInsert('@'));
    register_key(Input, "a".into(), command::InputInsert('a'));
    register_key(Input, "b".into(), command::InputInsert('b'));
    register_key(Input, "c".into(), command::InputInsert('c'));
    register_key(Input, "d".into(), command::InputInsert('d'));
    register_key(Input, "e".into(), command::InputInsert('e'));
    register_key(Input, "f".into(), command::InputInsert('f'));
    register_key(Input, "g".into(), command::InputInsert('g'));
    register_key(Input, "h".into(), command::InputInsert('h'));
    register_key(Input, "i".into(), command::InputInsert('i'));
    register_key(Input, "j".into(), command::InputInsert('j'));
    register_key(Input, "k".into(), command::InputInsert('k'));
    register_key(Input, "l".into(), command::InputInsert('l'));
    register_key(Input, "m".into(), command::InputInsert('m'));
    register_key(Input, "n".into(), command::InputInsert('n'));
    register_key(Input, "o".into(), command::InputInsert('o'));
    register_key(Input, "p".into(), command::InputInsert('p'));
    register_key(Input, "q".into(), command::InputInsert('q'));
    register_key(Input, "r".into(), command::InputInsert('r'));
    register_key(Input, "s".into(), command::InputInsert('s'));
    register_key(Input, "t".into(), command::InputInsert('t'));
    register_key(Input, "u".into(), command::InputInsert('u'));
    register_key(Input, "v".into(), command::InputInsert('v'));
    register_key(Input, "w".into(), command::InputInsert('w'));
    register_key(Input, "x".into(), command::InputInsert('x'));
    register_key(Input, "y".into(), command::InputInsert('y'));
    register_key(Input, "z".into(), command::InputInsert('z'));
    register_key(Input, "A".into(), command::InputInsert('A'));
    register_key(Input, "B".into(), command::InputInsert('B'));
    register_key(Input, "C".into(), command::InputInsert('C'));
    register_key(Input, "D".into(), command::InputInsert('D'));
    register_key(Input, "E".into(), command::InputInsert('E'));
    register_key(Input, "F".into(), command::InputInsert('F'));
    register_key(Input, "G".into(), command::InputInsert('G'));
    register_key(Input, "H".into(), command::InputInsert('H'));
    register_key(Input, "I".into(), command::InputInsert('I'));
    register_key(Input, "J".into(), command::InputInsert('J'));
    register_key(Input, "K".into(), command::InputInsert('K'));
    register_key(Input, "L".into(), command::InputInsert('L'));
    register_key(Input, "M".into(), command::InputInsert('M'));
    register_key(Input, "N".into(), command::InputInsert('N'));
    register_key(Input, "O".into(), command::InputInsert('O'));
    register_key(Input, "P".into(), command::InputInsert('P'));
    register_key(Input, "Q".into(), command::InputInsert('Q'));
    register_key(Input, "R".into(), command::InputInsert('R'));
    register_key(Input, "S".into(), command::InputInsert('S'));
    register_key(Input, "T".into(), command::InputInsert('T'));
    register_key(Input, "U".into(), command::InputInsert('U'));
    register_key(Input, "V".into(), command::InputInsert('V'));
    register_key(Input, "W".into(), command::InputInsert('W'));
    register_key(Input, "X".into(), command::InputInsert('X'));
    register_key(Input, "Y".into(), command::InputInsert('Y'));
    register_key(Input, "Z".into(), command::InputInsert('Z'));
    register_key(Input, "[".into(), command::InputInsert('['));
    register_key(Input, "\\".into(), command::InputInsert('\\'));
    register_key(Input, "]".into(), command::InputInsert(']'));
    register_key(Input, "^".into(), command::InputInsert('^'));
    register_key(Input, "_".into(), command::InputInsert('_'));
    register_key(Input, "`".into(), command::InputInsert('`'));
    register_key(Input, "{".into(), command::InputInsert('{'));
    register_key(Input, "|".into(), command::InputInsert('|'));
    register_key(Input, "}".into(), command::InputInsert('}'));
    register_key(Input, "~".into(), command::InputInsert('~'));

    if let Some(ref define) = crate::config::load().keymap {
        if let Some(normal) = define.normal_key_map() {
            normal
                .into_iter()
                .for_each(|(from, to)| register_key(Normal, from, to))
        }

        if let Some(visual) = define.visual_key_map() {
            visual
                .into_iter()
                .for_each(|(from, to)| register_key(Visual, from, to))
        }

        if let Some(input) = define.input_key_map() {
            input
                .into_iter()
                .for_each(|(from, to)| register_key(Input, from, to))
        }
    }
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
