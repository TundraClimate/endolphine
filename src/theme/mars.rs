use super::rgb;
use crate::scheme;

scheme! {
    fg: rgb("#B4B4B4"),
    fg_dark: rgb("#B4B4B4"),
    label: rgb("#646464"),
    bg: rgb("#0F0F1E"),
    bg_dark: rgb("#0F0F0F"),
    bar: rgb("#555555"),
    bar_dark: rgb("#373737"),
    path_picked: rgb("#96B2B2"),
    bar_text: rgb("#0A0A0A"),
    bar_text_light: rgb("#8C8C8C"),
    perm_ty: rgb("#DC0000"),
    perm_r: rgb("#A00000"),
    perm_w: rgb("#C86400"),
    perm_e: rgb("#C8C800"),
    row_file: rgb("#A02828"),
    row_dir: rgb("#D2C864"),
    row_symlink: rgb("#32D22D"),
    row_broken: rgb("#000000"),
    bsize: rgb("#C8C800"),
    mod_time: rgb("#D25A00"),
    select: rgb("#EB8C00"),
    row_cursor: rgb("#373737"),
    input: crossterm::style::Color::Reset,
    widget_fg: rgb("#FFFFFF"),
    widget_fg_dark: rgb("#FFFFFF"),
    widget_bg: rgb("#0F0F1E"),
    widget_bg_dark: rgb("#0F0F0F"),
    widget_cursor: rgb("#464646"),
    menu_tag: rgb("#329632"),
    search_sur: rgb("#55F0B4"),
}
