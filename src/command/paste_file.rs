use super::Command;
use crate::{app, clipboard, config, cursor, input, menu, misc};

pub struct AskPaste;

impl Command for AskPaste {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        input::use_f_mut(|i| {
            let config = config::load();
            let default_paste_input = if config.paste.default_overwrite {
                "y"
            } else {
                ""
            };

            i.enable(default_paste_input, Some("Paste".into()));

            if config.paste.force_mode {
                crate::handler::handle_input_mode(
                    i,
                    crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
                );
            } else {
                crate::log!("overwrite the same files? (y/Y)");
            };
        });

        Ok(())
    }
}

pub struct Paste {
    pub content: String,
    pub native: bool,
}

impl Command for Paste {
    fn run(&self) -> Result<(), crate::app::Error> {
        let files = if self.native {
            if !clipboard::is_cmd_installed() {
                crate::sys_log!(
                    "w",
                    "File paste failed: native command not installed, and config the native-clip is enabled"
                );
                crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");

                return Ok(());
            }

            match clipboard::read_clipboard_native("text/uri-list") {
                Ok(text) => text
                    .lines()
                    .filter_map(|f| f.strip_prefix("file://"))
                    .map(std::path::PathBuf::from)
                    .filter(|f| misc::exists_item(f))
                    .collect::<Vec<std::path::PathBuf>>(),
                Err(e) => {
                    crate::sys_log!("w", "Couldn't read a clipboard: {}", e.kind());
                    crate::log!("Paste failed: {}", e.kind());

                    return Ok(());
                }
            }
        } else {
            clipboard::read_clipboard()
                .split('\n')
                .map(std::path::PathBuf::from)
                .filter(|c| misc::exists_item(c))
                .collect::<Vec<_>>()
        };

        let current_path = app::get_path();
        let overwrite_mode = ["y", "Y", config::load().key.paste.to_string().as_str()]
            .contains(&self.content.as_str());

        for file in files.iter() {
            let Ok(metadata) = file.symlink_metadata() else {
                continue;
            };

            if !file.exists() && !metadata.is_symlink() {
                continue;
            }

            let copied_path = {
                let copied = current_path.join(misc::file_name(file));

                if copied == *file {
                    let stem = copied
                        .file_stem()
                        .map(|s| s.to_string_lossy())
                        .unwrap_or_default();
                    let suffix = config::load().paste.similar_file_suffix();
                    let added_suffix =
                        if let Some(extension) = copied.extension().map(|e| e.to_string_lossy()) {
                            format!("{}{}.{}", stem, suffix, extension)
                        } else {
                            format!("{}{}", stem, suffix)
                        };

                    current_path.join(added_suffix)
                } else {
                    copied
                }
            };

            if (metadata.is_file() || metadata.is_symlink())
                && (!misc::exists_item(&copied_path) || overwrite_mode)
            {
                if let Err(e) = std::fs::copy(file, &copied_path) {
                    crate::sys_log!("w", "Paste from clipboard failed: {}", e.kind());
                    crate::log!("Paste failed: \"{}\"", e.kind());
                }
            }

            if metadata.is_dir() {
                for entry in walkdir::WalkDir::new(file).into_iter().flatten() {
                    if entry.file_type().is_dir() {
                        continue;
                    }

                    let Ok(rel_path) = entry.path().strip_prefix(file) else {
                        continue;
                    };

                    let copied_path = copied_path.join(rel_path);

                    if !misc::exists_item(&copied_path) || overwrite_mode {
                        let parent = misc::parent(&copied_path);

                        if !parent.exists() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                crate::sys_log!("w", "Command Paste failed: {}", e.kind());
                                crate::log!("Paste failed: \"{}\"", e.kind());

                                continue;
                            }
                        }

                        if let Err(e) = std::fs::copy(entry.path(), &copied_path) {
                            crate::sys_log!("w", "Command Paste failed: {}", e.kind());
                            crate::log!("Paste failed: \"{}\"", e.kind());
                        }
                    }
                }
            }
        }

        cursor::load().resize(misc::child_files_len(&app::get_path()));
        crate::sys_log!("i", "Command Paste successful: {} files", files.len());
        crate::log!("{} files paste successful.", files.len());

        Ok(())
    }
}
