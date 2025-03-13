use super::rgb;
use crate::scheme;

scheme! {
    fg_focused: rgb("#FFFFFF"),
    fg_unfocused: rgb("#FFFFFF"),
    label: rgb("#646464"),
    bg_focused: rgb("#141414"),
    bg_unfocused: rgb("#000000"),
    bar: rgb("#FFFFFF"),
    bar_dark: rgb("#E6E6E6"),
    unnecessary_text: rgb("#969696"),
    bar_text: rgb("#282828"),
    bar_text_light: rgb("#646464"),
    perm_ty: rgb("#FF00FF"),
    perm_r: rgb("#FF0000"),
    perm_w: rgb("#00FF00"),
    perm_e: rgb("#0000FF"),
    row_file: rgb("#FF00FF"),
    row_dir: rgb("#00FF00"),
    row_symlink: rgb("#FFFF00"),
    row_broken: rgb("#000000"),
    row_bsize: rgb("#FF0000"),
    row_mod_time: rgb("#00FFFF"),
    select: rgb("#EB8C00"),
    row_cursor: rgb("#555555"),
    input: crossterm::style::Color::Reset,
    menu_tag: rgb("#00FFFF"),
    search_surround: rgb("#55F0B4"),
}
