use crate::scheme;
use crate::theme::rgb;

scheme! {
    fg_focused: rgb("#000000"),
    bg_focused: crossterm::style::Color::Reset,
    label: rgb("#C8C8C8"),
    bar: rgb("#696969"),
    unnecessary_text: rgb("#696969"),
    bar_text: rgb("#D7D7D7"),
    bar_text_light: rgb("#9B9B9B"),
    perm_ty: rgb("#E10519"),
    perm_r: rgb("#9B2369"),
    perm_w: rgb("#0F55B9"),
    perm_e: rgb("#0505C3"),
    row_file: rgb("#D723D7"),
    row_dir: rgb("#D73737"),
    row_symlink: rgb("#37AF37"),
    row_broken: rgb("#000000"),
    row_bsize: rgb("#0505C3"),
    row_mod_time: rgb("#7DB900"),
    select: rgb("#1473FF"),
    row_cursor: rgb("#AAAAAA"),
    menu_tag: rgb("#C80F4B"),
    search_surround: rgb("#AA0F4B"),
}
