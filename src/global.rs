use crate::{cursor::Cursor, error::*, input::Input, menu::Menu, misc};
use once_cell::sync::Lazy;
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU16, Ordering},
        RwLock,
    },
};

static PATH: Lazy<RwLock<PathBuf>> = Lazy::new(|| RwLock::new(PathBuf::new()));
static HEIGHT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(100));
static CURSOR: Lazy<Cursor> = Lazy::new(|| Cursor::new());
static VIEW_SHIFT: Lazy<AtomicU16> = Lazy::new(|| AtomicU16::new(0));
static MENU: Lazy<Menu> = Lazy::new(|| Menu::default());
static INPUT: Lazy<RwLock<Input>> = Lazy::new(|| RwLock::new(Input::default()));

pub fn init(path: &PathBuf) -> EpResult<()> {
    set_path(&path);

    let (_, row) = crossterm::terminal::size().map_err(|_| EpError::InitFailed)?;
    set_height(row);

    let c = misc::child_files_len(&path);
    CURSOR.resize(c);

    Ok(())
}

pub fn get_path() -> PathBuf {
    (*PATH.read().unwrap()).clone()
}

pub fn set_path(new_path: &PathBuf) {
    let mut lock = PATH.write().unwrap();
    *lock = new_path.clone();
}

pub fn get_height() -> u16 {
    HEIGHT.load(Ordering::Relaxed)
}

pub fn set_height(new_value: u16) {
    HEIGHT.swap(new_value, Ordering::Relaxed);
}

pub fn cursor() -> &'static Cursor {
    &*CURSOR
}

pub fn captured_cursor() -> &'static Cursor {
    if MENU.is_enabled() {
        MENU.cursor()
    } else {
        cursor()
    }
}

pub fn menu() -> &'static Menu {
    &*MENU
}

pub fn get_view_shift() -> u16 {
    VIEW_SHIFT.load(Ordering::Relaxed)
}

pub fn set_view_shift(new_value: u16) {
    VIEW_SHIFT.swap(new_value, Ordering::Relaxed);
}

pub fn input() -> &'static RwLock<Input> {
    &*INPUT
}

pub fn input_use<F: FnOnce(&Input) -> R, R>(f: F) -> R {
    let lock = input().read().unwrap();
    f(&*lock)
}

pub fn input_use_mut<F: FnOnce(&mut Input) -> R, R>(f: F) -> R {
    let mut lock = input().write().unwrap();
    f(&mut *lock)
}
