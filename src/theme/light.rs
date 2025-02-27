use crate::rgb;

pub const SCHEME: super::Scheme = super::Scheme {
    fg: rgb!(0),
    fg_dark: rgb!(0),
    bg: rgb!(195),
    bg_dark: rgb!(165),
    bar: rgb!(105),
    bar_dark: rgb!(135),
    path_picked: rgb!(105),
    bar_text: rgb!(215),
    bar_text_light: rgb!(155),
    perm_ty: rgb!(225, 5, 25),
    perm_r: rgb!(155, 35, 105),
    perm_w: rgb!(15, 85, 185),
    perm_e: rgb!(5, 5, 195),
    row_file: rgb!(215, 35, 215),
    row_dir: rgb!(215, 55, 55),
    row_symlink: rgb!(55, 175, 55),
    row_broken: rgb!(0),
    mod_time: rgb!(125, 185, 0),
    select: rgb!(20, 115, 255),
    row_cursor: rgb!(170),
    input: crossterm::style::Color::Reset,
    widget_fg: rgb!(0),
    widget_fg_dark: rgb!(0),
    widget_bg: rgb!(195),
    widget_bg_dark: rgb!(165),
    widget_cursor: rgb!(185),
    menu_tag: rgb!(200, 15, 75),
    search_sur: rgb!(170, 15, 75),
};
