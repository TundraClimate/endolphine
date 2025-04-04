use super::Command;
use crate::{app, clipboard, cursor, input, menu, misc};

pub struct AskDelete;

impl Command for AskDelete {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        if cursor::is_selection() {
            ask_rm_selected();

            return Ok(());
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            ask_rm_target(under_cursor_file);
        }

        Ok(())
    }
}

fn ask_rm_selected() {
    let selected_files = misc::sorted_child_files(&app::get_path())
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
        .collect::<Vec<_>>();

    input::use_f_mut(|i| i.enable("", Some("RmSelected".into())));
    crate::sys_log!("i", "Called command: RmSelected");
    crate::log!("Delete {} items ? (y/Y)", selected_files.len());
}

fn ask_rm_target(under_cursor_file: &std::path::Path) {
    input::use_f_mut(|i| i.enable("", Some("RmFileOrDir".into())));
    crate::sys_log!("i", "Called command: RmFileOrDir");
    crate::log!("Delete \"{}\" ? (y/Y)", misc::file_name(under_cursor_file));
}

fn yank(native: bool, paths: &[std::path::PathBuf]) {
    if native && !clipboard::is_cmd_installed() {
        crate::sys_log!(
            "w",
            "File yank failed: native command not installed, and config the native-clip is enabled"
        );
        crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

        return;
    }

    let tmp = std::path::Path::new("/tmp").join("endolphine");

    use std::fmt::Write;
    let text = paths.iter().fold(String::new(), |mut acc, p| {
        let _ = writeln!(
            acc,
            "{}{}",
            if native { "file://" } else { "" },
            tmp.join(misc::file_name(p)).to_string_lossy()
        );
        acc
    });

    if native {
        if let Err(e) = clipboard::clip_native(&text, "text/uri-list") {
            crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
            crate::log!("Yank failed: {}", e.kind());
        }
    } else {
        clipboard::clip(&text)
    }
}

pub struct RmFileOrDir {
    pub use_tmp: bool,
    pub yank_and_native: (bool, bool),
}

impl Command for RmFileOrDir {
    fn run(&self) -> Result<(), crate::app::Error> {
        let files = misc::sorted_child_files(&app::get_path());
        let Some(under_cursor_file) = files.get(cursor::load().current()) else {
            crate::sys_log!(
                "w",
                "Command RmFileOrDir failed: cursor in invalid position"
            );
            crate::log!("Delete file failed: target cannot find");

            return Ok(());
        };

        let Ok(metadata) = under_cursor_file.symlink_metadata() else {
            crate::sys_log!(
                "w",
                "Command RmFileOrDir failed: target metadata cannot access"
            );
            crate::log!("Delete file failed: cannot access metadata");

            return Ok(());
        };

        if !under_cursor_file.exists() && !metadata.is_symlink() {
            crate::sys_log!("w", "Command RmFileOrDir failed: target file not exists");
            crate::log!("Delete file failed: target not exists");

            return Ok(());
        }

        let name = misc::file_name(under_cursor_file);

        if self.use_tmp && self.yank_and_native.0 {
            yank(self.yank_and_native.1, &[under_cursor_file.to_path_buf()]);
        }

        let res = if self.use_tmp {
            misc::into_tmp(&[under_cursor_file.to_path_buf()])
        } else if under_cursor_file.is_dir() {
            misc::remove_dir_all(under_cursor_file)
        } else {
            std::fs::remove_file(under_cursor_file)
        };

        if let Err(e) = res {
            crate::sys_log!("w", "Command RmFileOrDir failed: {}", e.kind());
            crate::log!("Delete file failed: {}", e.kind());

            return Ok(());
        }

        cursor::load().resize(misc::child_files_len(&app::get_path()));
        crate::sys_log!(
            "i",
            "Command RmFileOrDir successful: delete the \"{}\"",
            name
        );
        crate::log!("\"{}\" delete successful", name);

        Ok(())
    }
}

pub struct RmSelected {
    pub use_tmp: bool,
    pub yank_and_native: (bool, bool),
}

impl Command for RmSelected {
    fn run(&self) -> Result<(), crate::app::Error> {
        let selected = misc::sorted_child_files(&app::get_path())
            .into_iter()
            .enumerate()
            .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
            .collect::<Vec<_>>();

        if self.use_tmp && self.yank_and_native.0 {
            yank(self.yank_and_native.1, &selected);
        }

        if self.use_tmp {
            if let Err(e) = misc::into_tmp(&selected) {
                crate::sys_log!("w", "Command RmSelected failed: {}", e.kind());
                crate::log!("Delete file failed: {}", e.kind());

                return Ok(());
            }
        } else {
            for target in &selected {
                let Ok(metadata) = target.symlink_metadata() else {
                    crate::sys_log!(
                        "w",
                        "Command RmSelected failed: target metadata cannot access"
                    );
                    crate::log!("Delete file failed: cannot access metadata");

                    return Ok(());
                };

                if !target.exists() && !metadata.is_symlink() {
                    crate::sys_log!("w", "Command RmSelected failed: target file not exists");
                    crate::log!("Delete file failed: target not exists");

                    return Ok(());
                }

                let res = if target.is_dir() {
                    misc::remove_dir_all(target)
                } else {
                    std::fs::remove_file(target)
                };

                if let Err(e) = res {
                    crate::sys_log!("w", "Command RmSelected failed: {}", e.kind());
                    crate::log!("Delete file failed: {}", e.kind());

                    return Ok(());
                }
            }
        }

        cursor::load().resize(misc::child_files_len(&app::get_path()));
        cursor::disable_selection();
        crate::sys_log!(
            "i",
            "Command RmSelected successful: {} files deleted",
            selected.len()
        );
        crate::log!("{} items delete successful", selected.len());

        Ok(())
    }
}
