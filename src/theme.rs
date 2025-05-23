use crossterm::style::Color;
use std::path::PathBuf;

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
    Bloom,
    Collapse,
}

macro_rules! schemes {
    ($($field:ident),+ $(,)?) => {
        pub struct Scheme {
            $(pub $field: Color),+
        }

        #[derive(serde::Deserialize, serde::Serialize)]
        pub struct SchemeWrap {
            $(pub $field: ColorWrap),+
        }
    }
}

pub struct ColorWrap {
    inner: String,
}

impl<'de> serde::Deserialize<'de> for ColorWrap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(ColorWrap { inner: s })
    }
}

impl serde::Serialize for ColorWrap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.inner)
    }
}

schemes! {
    fg_focused,
    fg_unfocused,
    bg_focused,
    bg_unfocused,
    label,
    bar,
    bar_dark,
    unnecessary_text,
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
    row_cursor,
    row_bsize,
    row_mod_time,
    select,
    input,
    menu_tag,
    search_surround,
}

impl From<ColorWrap> for Color {
    fn from(value: ColorWrap) -> Self {
        let s = &value.inner;

        if s.eq_ignore_ascii_case("RESET") {
            return Color::Reset;
        }

        rgb(s)
    }
}

impl From<SchemeWrap> for Scheme {
    fn from(value: SchemeWrap) -> Self {
        Self {
            fg_focused: value.fg_focused.into(),
            fg_unfocused: value.fg_unfocused.into(),
            bg_focused: value.bg_focused.into(),
            bg_unfocused: value.bg_unfocused.into(),
            label: value.label.into(),
            bar: value.bar.into(),
            bar_dark: value.bar_dark.into(),
            unnecessary_text: value.unnecessary_text.into(),
            bar_text: value.bar_text.into(),
            bar_text_light: value.bar_text_light.into(),
            perm_ty: value.perm_ty.into(),
            perm_r: value.perm_r.into(),
            perm_w: value.perm_w.into(),
            perm_e: value.perm_e.into(),
            row_file: value.row_file.into(),
            row_dir: value.row_dir.into(),
            row_symlink: value.row_symlink.into(),
            row_broken: value.row_broken.into(),
            row_cursor: value.row_cursor.into(),
            row_bsize: value.row_bsize.into(),
            row_mod_time: value.row_mod_time.into(),
            select: value.select.into(),
            input: value.input.into(),
            menu_tag: value.menu_tag.into(),
            search_surround: value.search_surround.into(),
        }
    }
}

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
        pub const SCHEME: std::sync::LazyLock<$crate::theme::Scheme> = std::sync::LazyLock::new(|| $crate::theme::Scheme {
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

pub fn scheme() -> &'static Scheme {
    crate::config::theme()
}

pub fn app_fg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().fg_unfocused
    } else {
        scheme().fg_focused
    }
}

pub fn app_bg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().bg_unfocused
    } else {
        scheme().bg_focused
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
        scheme().fg_focused
    } else {
        scheme().fg_unfocused
    }
}

pub fn widget_bg() -> Color {
    if crate::menu::refs().is_enabled() {
        scheme().bg_focused
    } else {
        scheme().bg_unfocused
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
        scheme().row_cursor
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
