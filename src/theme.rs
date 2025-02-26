use crate::global;
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
            $(pub $name: Color),+
        }
    }
}

scheme!(
    fg,
    fg_dark,
    bg,
    bg_dark,
    bar,
    bar_dark,
    path_picked,
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
    widget_fg,
    widget_fg_dark,
    widget_bg,
    widget_bg_dark,
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

pub fn app_fg() -> Color {
    if crate::global::menu().is_enabled() {
        global::color().fg_dark
    } else {
        global::color().fg
    }
}

pub fn app_bg() -> Color {
    if crate::global::menu().is_enabled() {
        global::color().bg_dark
    } else {
        global::color().bg
    }
}

pub fn bar_color() -> Color {
    if crate::global::menu().is_enabled() {
        global::color().bar_dark
    } else {
        global::color().bar
    }
}

pub fn widget_fg() -> Color {
    if crate::global::menu().is_enabled() {
        global::color().widget_fg
    } else {
        global::color().widget_fg_dark
    }
}

pub fn widget_bg() -> Color {
    if crate::global::menu().is_enabled() {
        global::color().widget_bg
    } else {
        global::color().widget_bg_dark
    }
}

pub fn path_name(path: &PathBuf) -> Color {
    match path {
        path if !path.exists() => global::color().row_broken,
        path if path.is_symlink() => global::color().row_symlink,
        path if path.is_dir() => global::color().row_dir,
        path if path.is_file() => global::color().row_file,
        _ => global::color().row_broken,
    }
}

pub fn item_bg(is_selected: bool, is_cursor_pos: bool) -> Color {
    if is_selected {
        global::color().select
    } else if is_cursor_pos {
        global::color().row_cursor
    } else {
        app_bg()
    }
}

pub fn widget_item_bg(is_cursor_pos: bool, is_enable: bool) -> Color {
    if is_cursor_pos && is_enable {
        global::color().widget_cursor
    } else {
        widget_bg()
    }
}

pub fn permission(index: usize) -> Color {
    match index % 3 {
        0 => global::color().perm_r,
        1 => global::color().perm_w,
        _ => global::color().perm_e,
    }
}
