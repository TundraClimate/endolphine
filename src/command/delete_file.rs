use super::Command;
use crate::{app, clipboard, config, cursor, input, menu, misc};

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

pub struct RmFileOrDir {
    pub content: String,
    pub native: bool,
}

impl Command for RmFileOrDir {
    fn run(&self) -> Result<(), crate::app::Error> {
        if !["y", "Y", config::load().key.delete.to_string().as_str()]
            .contains(&self.content.as_str())
        {
            return Ok(());
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor::load().current())
        {
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

            let res = if config::load().rm.for_tmp {
                if config::load().rm.yank {
                    if self.native && !clipboard::is_cmd_installed() {
                        crate::sys_log!(
                            "w",
                            "File yank failed: native command not installed, and config the native-clip is enabled"
                        );
                        crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

                        return Ok(());
                    }

                    let tmp_path = std::path::Path::new("/tmp")
                        .join("endolphine")
                        .join(misc::file_name(under_cursor_file));

                    if self.native {
                        if let Err(e) = clipboard::clip_native(
                            &format!("file://{}", tmp_path.to_string_lossy()),
                            "text/uri-list",
                        ) {
                            crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                            crate::log!("Yank failed: {}", e.kind());
                        }
                    } else {
                        clipboard::clip(&tmp_path.to_string_lossy());
                    }
                }
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
        } else {
            crate::sys_log!(
                "w",
                "Command RmFileOrDir failed: cursor in invalid position"
            );
            crate::log!("Delete file failed: target cannot find");
        }

        Ok(())
    }
}

pub struct RmSelected {
    pub content: String,
    pub native: bool,
}

impl Command for RmSelected {
    fn run(&self) -> Result<(), crate::app::Error> {
        if !["y", "Y", config::load().key.delete.to_string().as_str()]
            .contains(&self.content.as_str())
        {
            return Ok(());
        }

        let selected = misc::sorted_child_files(&app::get_path())
            .into_iter()
            .enumerate()
            .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
            .collect::<Vec<_>>();

        if config::load().rm.for_tmp {
            if config::load().rm.yank {
                if self.native && !clipboard::is_cmd_installed() {
                    crate::sys_log!(
                        "w",
                        "File yank failed: native command not installed, and config the native-clip is enabled"
                    );
                    crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

                    return Ok(());
                }

                let tmp = std::path::Path::new("/tmp").join("endolphine");

                use std::fmt::Write;
                let text = selected.iter().fold(String::new(), |mut acc, p| {
                    let _ = writeln!(
                        acc,
                        "{}{}",
                        if self.native { "file://" } else { "" },
                        tmp.join(misc::file_name(p)).to_string_lossy()
                    );
                    acc
                });

                if self.native {
                    if let Err(e) = clipboard::clip_native(&text, "text/uri-list") {
                        crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                        crate::log!("Yank failed: {}", e.kind());
                    }
                } else {
                    clipboard::clip(&text)
                }
            }

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
