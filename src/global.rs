use crate::{
    config::Config, cursor::Cursor, error::*, input::Input, menu::Menu, misc, theme::Scheme,
};
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
};

static RENDERING: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static PATH: Lazy<RwLock<PathBuf>> = Lazy::new(|| RwLock::new(PathBuf::new()));
static CANVAS_SIZE: Lazy<(AtomicU16, AtomicU16)> =
    Lazy::new(|| (AtomicU16::new(100), AtomicU16::new(100)));
static CURSOR: Lazy<Cursor> = Lazy::new(Cursor::new);
static VIEW_SHIFT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(0));
static MENU: Lazy<Menu> = Lazy::new(Menu::default);
static INPUT: Lazy<RwLock<Input>> = Lazy::new(|| RwLock::new(Input::default()));
static CACHE: Lazy<RwLock<HashMap<(u16, u8), String>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static MATCHER_TEXT: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new(String::new()));
static PROCESS_COUNT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(0));
static CONFIG: Lazy<Config> = Lazy::new(Config::load);

pub fn init(path: &Path) -> EpResult<()> {
    set_path(path);

    let (width, height) =
        crossterm::terminal::size().map_err(|e| EpError::Init(e.kind().to_string()))?;
    set_width(width);
    set_height(height);

    let c = misc::child_files_len(path);
    CURSOR.resize(c);

    Ok(())
}

pub fn is_render() -> bool {
    RENDERING.load(Ordering::Relaxed)
}

pub fn disable_render() {
    RENDERING.swap(false, Ordering::Relaxed);
}

pub fn enable_render() {
    RENDERING.swap(true, Ordering::Relaxed);
}

pub fn get_path() -> PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: &Path) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path.to_path_buf();
}

pub fn get_width() -> u16 {
    CANVAS_SIZE.0.load(Ordering::Relaxed)
}

pub fn set_width(new_value: u16) {
    CANVAS_SIZE.0.swap(new_value, Ordering::Relaxed);
}

pub fn get_height() -> u16 {
    CANVAS_SIZE.1.load(Ordering::Relaxed)
}

pub fn set_height(new_value: u16) {
    CANVAS_SIZE.1.swap(new_value, Ordering::Relaxed);
}

pub fn cursor() -> &'static Cursor {
    &CURSOR
}

pub fn captured_cursor() -> &'static Cursor {
    if MENU.is_enabled() {
        MENU.cursor()
    } else {
        cursor()
    }
}

pub fn menu() -> &'static Menu {
    &MENU
}

pub fn get_view_shift() -> u16 {
    VIEW_SHIFT.load(Ordering::Relaxed)
}

pub fn set_view_shift(new_value: u16) {
    VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
}

pub fn input() -> &'static RwLock<Input> {
    &INPUT
}

pub fn input_use<F: FnOnce(&Input) -> R, R>(f: F) -> R {
    let lock = input().read().unwrap();
    f(&lock)
}

pub fn input_use_mut<F: FnOnce(&mut Input) -> R, R>(f: F) -> R {
    let mut lock = input().write().unwrap();
    f(&mut lock)
}

pub fn cache_insert(key: (u16, u8), tag: String) {
    CACHE.write().unwrap().insert(key, tag);
}

pub fn cache_match(key: (u16, u8), tag: &str) -> bool {
    CACHE.read().unwrap().get(&key).map(|c| c.as_ref()) == Some(tag)
}

pub fn cache_clear() {
    CACHE.write().unwrap().clear();
}

pub fn matcher_update<F: FnOnce(&mut String)>(f: F) {
    let mut lock = MATCHER_TEXT.write().unwrap();
    f(&mut lock);
}

pub fn is_match_text<F: FnOnce(&str) -> bool>(f: F) -> bool {
    let lock = MATCHER_TEXT.read().unwrap();
    f(&lock)
}

pub fn read_matcher() -> String {
    let lock = MATCHER_TEXT.read().unwrap();
    lock.to_owned()
}

pub fn proc_count_up() {
    PROCESS_COUNT.store(PROCESS_COUNT.load(Ordering::Relaxed) + 1, Ordering::Relaxed);
}

pub fn proc_count_down() {
    PROCESS_COUNT.store(PROCESS_COUNT.load(Ordering::Relaxed) - 1, Ordering::Relaxed);
}

pub fn procs() -> u16 {
    PROCESS_COUNT.load(Ordering::Relaxed)
}

pub fn config() -> &'static Config {
    &CONFIG
}

pub fn color() -> Scheme {
    config().scheme()
}
