use crate::{cursor::Cursor, global};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const MENU_LENGTH: u16 = 20;

pub fn toggle_open() {
    if is_opened() {
        global::set_view_shift(0);
    } else {
        global::set_view_shift(MENU_LENGTH);
    }
}

pub fn is_opened() -> bool {
    let shift = global::get_view_shift();
    shift == MENU_LENGTH
}

pub struct Menu {
    elements: Vec<MenuElement>,
    cursor: Cursor,
    enable: AtomicBool,
}

impl Default for Menu {
    fn default() -> Self {
        let home_path = option_env!("HOME").unwrap_or("/root");
        let dls_path = format!("{}/Downloads", home_path);
        let desktop_path = format!("{}/Desktop", home_path);

        Menu {
            elements: vec![
                MenuElement::new("Home", home_path),
                MenuElement::new("Downloads", dls_path),
                MenuElement::new("Desktop", desktop_path),
            ],
            cursor: Cursor::new(),
            enable: AtomicBool::new(false),
        }
    }
}

impl Menu {
    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn is_enabled(&self) -> bool {
        self.enable.load(Ordering::Relaxed)
    }

    pub fn resize_cursor(&self) {
        self.cursor().resize(self.elements.len());
    }

    pub fn elements(&self) -> Vec<MenuElement> {
        self.elements.clone()
    }

    pub fn toggle_enable(&self) {
        self.resize_cursor();
        if self.is_enabled() {
            self.enable.swap(false, Ordering::Relaxed);
        } else {
            self.enable.swap(true, Ordering::Relaxed);
        }
    }
}

#[derive(Clone)]
pub struct MenuElement {
    tag: String,
    path: PathBuf,
}

impl MenuElement {
    fn new(tag: &str, path: impl AsRef<Path>) -> Self {
        MenuElement {
            tag: String::from(tag),
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn tag(&self) -> String {
        self.tag.clone()
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}
