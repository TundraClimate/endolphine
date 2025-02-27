use crate::{cursor::Cursor, global};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    sync::atomic::{AtomicBool, Ordering},
};

global!(MENU<Menu>, Menu::default, {
    pub fn menu() -> &'static Menu {
        &MENU
    }
});

const MENU_LENGTH: u16 = 20;

pub fn toggle_open() {
    if is_opened() {
        crate::canvas::set_view_shift(0);
    } else {
        crate::canvas::set_view_shift(MENU_LENGTH);
    }
}

pub fn is_opened() -> bool {
    let shift = crate::canvas::get_view_shift();
    shift == MENU_LENGTH
}

pub struct Menu {
    elements: Vec<MenuElement>,
    cursor: Cursor,
    enable: AtomicBool,
}

impl Default for Menu {
    fn default() -> Self {
        Menu {
            elements: crate::config::config().menu.items.clone(),
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

    pub fn elements(&self) -> &Vec<MenuElement> {
        &self.elements
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
    pub fn new(tag: &str, path: impl AsRef<Path>) -> Self {
        MenuElement {
            tag: String::from(tag),
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl FromStr for MenuElement {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((tag, path)) = s.trim().split_once(":") {
            Ok(MenuElement::new(tag, path))
        } else {
            let path = Path::new(s.trim());
            let Some(tag) = path.file_name().map(|name| name.to_string_lossy()) else {
                return Err(String::from("invalid string"));
            };
            Ok(MenuElement::new(&tag, path))
        }
    }
}

impl serde::ser::Serialize for MenuElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.tag, self.path.to_string_lossy()))
    }
}

impl<'de> serde::Deserialize<'de> for MenuElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        MenuElement::from_str(&s).map_err(|e| serde::de::Error::custom(&e))
    }
}
