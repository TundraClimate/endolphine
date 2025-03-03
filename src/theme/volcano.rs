use super::rgb;
use crate::scheme;

scheme!(
    fg: rgb("#DCA0A0"),
    fg_dark: rgb("#B48C8C"),
    label: rgb("#280000"),
    bg: rgb("#281414"),
    bg_dark: rgb("#140000"),
    bar: rgb("#DC1414"),
    bar_dark: rgb("#B40000"),
    path_picked: rgb("#96A0A0"),
    bar_text: rgb("#282828"),
    bar_text_light: rgb("#646464"),
    perm_ty: rgb("#C81414"),
    perm_r: rgb("#C83C14"),
    perm_w: rgb("#C87814"),
    perm_e: rgb("#C8B414"),
    row_file: rgb("#FF9B00"),
    row_dir: rgb("#9B0000"),
    row_symlink: rgb("#C828C8"),
    row_broken: rgb("#000000"),
    bsize: rgb("#B4B428"),
    mod_time: rgb("#B45000"),
    select: rgb("#EB8C00"),
    row_cursor: rgb("#555555"),
    input: crossterm::style::Color::Reset,
    widget_fg: rgb("#FFFFFF"),
    widget_fg_dark: rgb("#FFFFFF"),
    widget_bg: rgb("#281414"),
    widget_bg_dark: rgb("#140000"),
    widget_cursor: rgb("#3F3F3F"),
    menu_tag: rgb("#F05050"),
    search_sur: rgb("#EB8C00"),
);
