use crate::rgb;

pub const SCHEME: super::Scheme = super::Scheme {
    fg: rgb!(255),
    fg_dark: rgb!(255),
    label: rgb!(100),
    bg: rgb!(60),
    bg_dark: rgb!(30),
    bar: rgb!(150),
    bar_dark: rgb!(120),
    path_picked: rgb!(150),
    bar_text: rgb!(40),
    bar_text_light: rgb!(100),
    perm_ty: rgb!(30, 250, 230),
    perm_r: rgb!(100, 220, 150),
    perm_w: rgb!(240, 170, 70),
    perm_e: rgb!(250, 250, 60),
    row_file: rgb!(40, 220, 40),
    row_dir: rgb!(40, 200, 200),
    row_symlink: rgb!(200, 40, 200),
    row_broken: rgb!(200, 0, 0),
    bsize: rgb!(250, 250, 60),
    mod_time: rgb!(130, 70, 255),
    select: rgb!(235, 140, 0),
    row_cursor: rgb!(85),
    input: crossterm::style::Color::Reset,
    widget_fg: rgb!(255),
    widget_fg_dark: rgb!(255),
    widget_bg: rgb!(90),
    widget_bg_dark: rgb!(50),
    widget_cursor: rgb!(70),
    menu_tag: rgb!(85, 240, 180),
    search_sur: rgb!(85, 240, 180),
};
