use chrono::{DateTime, Local};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};
use std::{io, path::PathBuf};
use unicode_segmentation::UnicodeSegmentation;

pub fn render(
    file: &PathBuf,
    is_menu: bool,
    is_cursor: bool,
    is_selected: bool,
    cols: u16,
) -> io::Result<()> {
    let file_names = crate::filename(&file).chars().take(65).collect::<String>();
    let file_len = file_names.graphemes(true).count();
    let pad = (file_names.len() - file_len) / 2;
    let select = if is_selected {
        Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        }
    } else {
        Color::Reset
    };
    execute!(
        io::stdout(),
        SetBackgroundColor(select),
        Print(if is_cursor { "~>" } else { "  " }),
        Print(" | "),
        SetForegroundColor(colored_path(file)),
        Print(&file_names),
        ResetColor,
        SetBackgroundColor(select),
        Print(" ".repeat(cols as usize - file_len - pad - 27)),
        Print("| "),
        SetForegroundColor(Color::DarkBlue),
        Print(info_block(file, is_menu)?),
        ResetColor,
        Print(" |"),
        Print(if is_cursor { " <" } else { "  " }),
    )?;
    Ok(())
}

fn colored_path(file: &PathBuf) -> Color {
    if file.is_symlink() {
        Color::Magenta
    } else if file.is_dir() {
        Color::Green
    } else if file.is_file() {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn info_block(file: &PathBuf, is_menu: bool) -> io::Result<String> {
    Ok(if let Ok(meta) = file.metadata() {
        let datetime = DateTime::<Local>::from(meta.modified()?);
        datetime.format("%Y/%m/%d %H:%M").to_string()
    } else if is_menu {
        String::from(" Open to Select ")
    } else {
        String::from("       N/A      ")
    })
}
