use super::Command;
use crate::{app, clipboard, cursor, menu, misc};

pub struct Yank {
    pub native: bool,
}

impl Command for Yank {
    fn run(&self) -> Result<(), crate::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        if self.native {
            yank_native(cursor);
        } else {
            yank_local(cursor);
        }

        Ok(())
    }
}

fn yank_native(cursor: &cursor::Cursor) {
    if !clipboard::is_cmd_installed() {
        crate::sys_log!(
            "w",
            "File yank failed: native command not installed, and the native-clip config is enabled"
        );
        crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

        return;
    }

    if cursor::is_selection() {
        yank_selection_native();

        return;
    }

    if let Some(under_cursor_file) =
        misc::sorted_child_files(&app::get_path()).get(cursor.current())
    {
        yank_target_native(under_cursor_file);
    }
}

fn yank_selection_native() {
    let selected_files = misc::sorted_child_files(&app::get_path())
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
        .map(|f| format!("file://{}", f.to_string_lossy()))
        .collect::<Vec<_>>();

    if let Err(e) = clipboard::clip_native(&selected_files.join("\n"), "text/uri-list") {
        crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
        crate::log!("Yank failed: {}", e.kind());

        return;
    }

    cursor::disable_selection();
    crate::sys_log!("i", "{} files yanked", selected_files.len());
    crate::log!("Yanked {} items", selected_files.len());
}

fn yank_target_native(under_cursor_file: &std::path::Path) {
    let text = format!("file://{}", under_cursor_file.to_string_lossy());

    if let Err(e) = clipboard::clip_native(&text, "text/uri-list") {
        crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
        crate::log!("Yank failed: {}", e.kind());

        return;
    }

    crate::sys_log!(
        "i",
        "File the {} yanked",
        under_cursor_file.to_string_lossy()
    );
    crate::log!("Yanked \"{}\"", misc::file_name(under_cursor_file));
}

fn yank_local(cursor: &cursor::Cursor) {
    if cursor::is_selection() {
        yank_selection_local();

        return;
    }

    if let Some(under_cursor_file) =
        misc::sorted_child_files(&app::get_path()).get(cursor.current())
    {
        yank_target_local(under_cursor_file);
    }
}

fn yank_selection_local() {
    let selected_files = misc::sorted_child_files(&app::get_path())
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| cursor::is_selected(i).then_some(f.to_string_lossy().to_string()))
        .collect::<Vec<_>>();

    clipboard::clip(&selected_files.join("\n"));

    cursor::disable_selection();
    crate::sys_log!("i", "{} files yanked", selected_files.len());
    crate::log!("Yanked {} items", selected_files.len());
}

fn yank_target_local(under_cursor_file: &std::path::Path) {
    clipboard::clip(&under_cursor_file.to_string_lossy());

    crate::sys_log!(
        "i",
        "File the {} yanked",
        under_cursor_file.to_string_lossy()
    );
    crate::log!("Yanked \"{}\"", misc::file_name(under_cursor_file));
}
