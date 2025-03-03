use crate::rgb;

pub const SCHEME: super::Scheme = super::Scheme {
    fg: rgb!(180),
    fg_dark: rgb!(160),
    label: rgb!(0, 0, 40),
    bg: rgb!(20),
    bg_dark: rgb!(0),
    bar: rgb!(60, 60, 180),
    bar_dark: rgb!(20, 20, 140),
    path_picked: rgb!(120),
    bar_text: rgb!(220),
    bar_text_light: rgb!(200),
    perm_ty: rgb!(100, 100, 160),
    perm_r: rgb!(100, 100, 190),
    perm_w: rgb!(100, 100, 220),
    perm_e: rgb!(100, 100, 250),
    row_file: rgb!(150, 150, 255),
    row_dir: rgb!(50, 50, 255),
    row_symlink: rgb!(0, 0, 255),
    row_broken: rgb!(120, 0, 0),
    bsize: rgb!(90, 90, 220),
    mod_time: rgb!(70, 70, 255),
    select: rgb!(0, 140, 235),
    row_cursor: rgb!(60, 60, 85),
    input: crossterm::style::Color::Reset,
    widget_fg: rgb!(255),
    widget_fg_dark: rgb!(255),
    widget_bg: rgb!(20),
    widget_bg_dark: rgb!(0),
    widget_cursor: rgb!(70),
    menu_tag: rgb!(85, 180, 240),
    search_sur: rgb!(85, 180, 240),
};
