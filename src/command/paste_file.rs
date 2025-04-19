use super::Command;
use crate::{app, clipboard, config, cursor, input, menu, misc};

pub struct AskPaste;

impl Command for AskPaste {
    fn run(&self) -> Result<(), crate::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let config = config::load();

        if config.paste.force_mode {
            Paste {
                overwrite: config.paste.default_overwrite,
                native: config::load().native_clip,
            }
            .run()?;

            return Ok(());
        };

        let default_paste_input = if config.paste.default_overwrite {
            "y"
        } else {
            ""
        };

        input::enable(default_paste_input, Some("Paste".into()));

        crate::log!("overwrite the same files? (y/Y)");

        Ok(())
    }
}

pub struct Paste {
    pub overwrite: bool,
    pub native: bool,
}

impl Command for Paste {
    fn run(&self) -> Result<(), crate::Error> {
        let Some(files) = read_clipboard(self.native) else {
            return Ok(());
        };

        let current_path = app::get_path();

        for file in files.iter() {
            let Ok(metadata) = file.symlink_metadata() else {
                continue;
            };

            if !file.exists() && !metadata.is_symlink() {
                continue;
            }

            let copied_path = conflict_fix(&current_path, file);

            if (metadata.is_file() || metadata.is_symlink())
                && (!misc::exists_item(&copied_path) || self.overwrite)
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

                    if !misc::exists_item(&copied_path) || self.overwrite {
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

fn read_clipboard(native: bool) -> Option<Vec<std::path::PathBuf>> {
    if native {
        if !clipboard::is_cmd_installed() {
            crate::sys_log!(
                "w",
                "File paste failed: native command not installed, and config the native-clip is enabled"
            );
            crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");

            return None;
        }

        match clipboard::read_clipboard_native("text/uri-list") {
            Ok(text) => Some(
                text.lines()
                    .filter_map(|f| f.strip_prefix("file://"))
                    .map(std::path::PathBuf::from)
                    .filter(|f| misc::exists_item(f))
                    .collect::<Vec<std::path::PathBuf>>(),
            ),
            Err(e) => {
                crate::sys_log!("w", "Couldn't read a clipboard: {}", e.kind());
                crate::log!("Paste failed: {}", e.kind());

                None
            }
        }
    } else {
        Some(
            clipboard::read_clipboard()
                .split('\n')
                .map(std::path::PathBuf::from)
                .filter(|c| misc::exists_item(c))
                .collect::<Vec<_>>(),
        )
    }
}

fn conflict_fix(current_path: &std::path::Path, file: &std::path::Path) -> std::path::PathBuf {
    let copied = current_path.join(misc::file_name(file));

    if copied == *file {
        let stem = copied
            .file_stem()
            .map(|s| s.to_string_lossy())
            .unwrap_or_default();
        let suffix = config::load().paste.similar_file_suffix();
        let added_suffix = if let Some(extension) = copied.extension().map(|e| e.to_string_lossy())
        {
            format!("{}{}.{}", stem, suffix, extension)
        } else {
            format!("{}{}", stem, suffix)
        };

        current_path.join(added_suffix)
    } else {
        copied
    }
}
