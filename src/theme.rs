use crossterm::style::Color;
use std::path::PathBuf;

pub mod dark;
pub mod dark_no_bg;
pub mod holiday;
pub mod ice;
pub mod light;
pub mod light_no_bg;
pub mod mars;
pub mod monochrome;
pub mod mossy;
pub mod neon;
pub mod nept;
pub mod volcano;

#[derive(serde::Deserialize, serde::Serialize)]
pub enum Theme {
    Dark,
    DarkNoBg,
    Light,
    LightNoBg,
    Mars,
    Neon,
    Ice,
    Nept,
    Volcano,
    Mossy,
    Monochrome,
    Holiday,
}

macro_rules! colors {
    ($v:vis struct $name:ident { $($field:ident),+ $(,)? }) => {
        $v struct $name {
            $(pub $field: Color),+
        }
    }
}

colors!(pub struct Scheme {
    fg,
    fg_dark,
    bg,
    bg_dark,
    label,
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
    bsize,
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
});

impl From<std::sync::LazyLock<Scheme>> for Scheme {
    fn from(value: std::sync::LazyLock<Scheme>) -> Self {
        let val = &*value;
        Scheme { ..*val }
    }
}

#[macro_export]
macro_rules! scheme {
    ($($name:ident : $value:expr),* $(,)?) => {
        #[allow(clippy::declare_interior_mutable_const)]
        pub const SCHEME: std::sync::LazyLock<super::Scheme> = std::sync::LazyLock::new(|| super::Scheme {
            $($name: $value),*
        });
    }
}

pub fn rgb(t: &str) -> Color {
    if t.len() != 7 || !t.starts_with("#") {
        panic!("Invalid scheme: {}", t);
    }

    let r = u8::from_str_radix(&t[1..=2], 16).unwrap();
    let g = u8::from_str_radix(&t[3..=4], 16).unwrap();
    let b = u8::from_str_radix(&t[5..], 16).unwrap();

    Color::Rgb { r, g, b }
}

pub fn scheme() -> Scheme {
    crate::config::load().scheme()
}

pub fn app_fg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().fg_dark
    } else {
        scheme().fg
    }
}

pub fn app_bg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().bg_dark
    } else {
        scheme().bg
    }
}

pub fn bar_color() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().bar_dark
    } else {
        scheme().bar
    }
}

pub fn wid_bar_color() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().bar
    } else {
        scheme().bar_dark
    }
}

pub fn widget_fg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().widget_fg
    } else {
        scheme().widget_fg_dark
    }
}

pub fn widget_bg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().widget_bg
    } else {
        scheme().widget_bg_dark
    }
}

pub fn path_name(path: &PathBuf) -> Color {
    match path {
        path if !path.exists() => scheme().row_broken,
        path if path.is_symlink() => scheme().row_symlink,
        path if path.is_dir() => scheme().row_dir,
        path if path.is_file() => scheme().row_file,
        _ => scheme().row_broken,
    }
}

pub fn item_bg(is_selected: bool, is_cursor_pos: bool) -> Color {
    if is_selected {
        scheme().select
    } else if is_cursor_pos {
        scheme().row_cursor
    } else {
        app_bg()
    }
}

pub fn widget_item_bg(is_cursor_pos: bool, is_enable: bool) -> Color {
    if is_cursor_pos && is_enable {
        scheme().widget_cursor
    } else {
        widget_bg()
    }
}

pub fn permission(index: usize) -> Color {
    match index % 3 {
        0 => scheme().perm_r,
        1 => scheme().perm_w,
        _ => scheme().perm_e,
    }
}
