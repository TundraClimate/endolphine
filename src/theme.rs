use crossterm::style::Color;
use std::path::PathBuf;

pub mod dark;
pub mod light;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Theme {
    Dark,
    Light,
}

macro_rules! scheme {
    ($($name:ident),+ $(,)?) => {
        pub struct Scheme {
            $($name: Color),+
        }
    }
}

scheme!(
    bg,
    bg_dark,
    bar,
    bar_dark,
    current_path,
    bar_text,
    bar_text_light,
    perm_ty,
    perm_r,
    perm_w,
    perm_e,
    row_file,
    row_dir,
    row_symlink,
    row_broken,
    mod_time,
    select,
    row_cursor,
    input,
    widget,
    widget_dark,
    widget_cursor,
    menu_tag,
    search_sur,
);

#[macro_export]
macro_rules! rgb {
    ($r:expr, $g:expr, $b:expr) => {
        crossterm::style::Color::Rgb {
            r: $r,
            g: $g,
            b: $b,
        }
    };

    ($color:expr) => {
        crossterm::style::Color::Rgb {
            r: $color,
            g: $color,
            b: $color,
        }
    };
}

macro_rules! const_color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        pub const $name: Color = Color::Rgb {
            r: $r,
            g: $g,
            b: $b,
        };
    };

    ($name:ident, $color:expr) => {
        pub const $name: Color = Color::Rgb {
            r: $color,
            g: $color,
            b: $color,
        };
    };
}

const_color!(APP_BG, 60);
const_color!(APP_BG_DARK, 30);
const_color!(BAR, 150);
const_color!(BAR_DARK, 120);
const_color!(HEADER_CURRENT_PATH_ON_DARK, 150);
const_color!(HEADER_BAR_TEXT_DEFAULT, 40);
const_color!(HEADER_BAR_TEXT_LIGHT, 100);
const_color!(PERMISSION_TYPE, 30, 150, 230);
const_color!(PERMISSION_READ, 100, 220, 150);
const_color!(PERMISSION_WRITE, 240, 170, 70);
const_color!(PERMISSION_EXE, 250, 250, 60);
const_color!(PATH_NAME_FILE, 40, 220, 40);
const_color!(PATH_NAME_DIRECTORY, 40, 200, 200);
const_color!(PATH_NAME_SYMLINK, 200, 40, 200);
const_color!(PATH_NAME_BROKEN, 200, 0, 0);
const_color!(LAST_MODIFIED_TIME, 130, 70, 255);
const_color!(SELECTED, 235, 140, 0);
const_color!(UNDER_CURSOR, 85);
const_color!(INPUT_BG, 40, 40, 80);
const_color!(MENU_BG, 90);
const_color!(MENU_BG_DARK, 50);
const_color!(MENU_UNDER_CURSOR, 70);
const_color!(MENU_TAG_COLOR, 85, 240, 180);
const_color!(FILENAME_SURROUND, 100);

pub fn app_bg() -> Color {
    if crate::global::menu().is_enabled() {
        APP_BG_DARK
    } else {
        APP_BG
    }
}

pub fn bar_color() -> Color {
    if crate::global::menu().is_enabled() {
        BAR_DARK
    } else {
        BAR
    }
}

pub fn menu_bg() -> Color {
    if crate::global::menu().is_enabled() {
        MENU_BG
    } else {
        MENU_BG_DARK
    }
}

pub fn path_name(path: &PathBuf) -> Color {
    match path {
        path if !path.exists() => PATH_NAME_BROKEN,
        path if path.is_symlink() => PATH_NAME_SYMLINK,
        path if path.is_dir() => PATH_NAME_DIRECTORY,
        path if path.is_file() => PATH_NAME_FILE,
        _ => PATH_NAME_BROKEN,
    }
}

pub fn item_bg(is_selected: bool, is_cursor_pos: bool) -> Color {
    if is_selected {
        SELECTED
    } else if is_cursor_pos {
        UNDER_CURSOR
    } else {
        app_bg()
    }
}

pub fn menu_item_bg(is_cursor_pos: bool, is_enable: bool) -> Color {
    if is_cursor_pos && is_enable {
        MENU_UNDER_CURSOR
    } else {
        menu_bg()
    }
}

pub fn permission(index: usize) -> Color {
    match index % 3 {
        0 => PERMISSION_READ,
        1 => PERMISSION_WRITE,
        _ => PERMISSION_EXE,
    }
}
