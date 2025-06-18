use crate::scheme;
use crate::theme::rgb;

scheme! {
    fg_focused: rgb("#FFFFFF"),
    bg_focused: crossterm::style::Color::Reset,
    label: rgb("#646464"),
    bar: rgb("#969696"),
    unnecessary_text: rgb("#969696"),
    bar_text: rgb("#282828"),
    bar_text_light: rgb("#646464"),
    perm_ty: rgb("#1EFAE6"),
    perm_r: rgb("#64DC96"),
    perm_w: rgb("#F0AA46"),
    perm_e: rgb("#FAFA3C"),
    row_file: rgb("#28DC28"),
    row_dir: rgb("#28C8C8"),
    row_symlink: rgb("#C828C8"),
    row_broken: rgb("#C80000"),
    row_bsize: rgb("#FAFA3C"),
    row_mod_time: rgb("#8246FF"),
    select: rgb("#EB8C00"),
    row_cursor: rgb("#555555"),
    menu_tag: rgb("#55F0B4"),
    search_surround: rgb("#55F0B4"),
}
