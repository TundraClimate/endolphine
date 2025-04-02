use crate::{
    app, canvas, clipboard, config, cursor,
    input::{self, Input},
    misc,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};

pub fn handle_event() -> Result<bool, app::Error> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return handle_key_event(key),
            Event::Resize(_, _) => {
                cursor::load().resize(misc::child_files_len(&app::get_path()));
                canvas::cache_clear();
            }
            _ => {}
        }
    }

    Ok(false)
}

fn handle_key_event(key: KeyEvent) -> Result<bool, app::Error> {
    if input::use_f(|i| i.is_enable()) {
        input::use_f_mut(|i| handle_input_mode(i, key));

        return Ok(false);
    }

    {
        let key = crate::key::Key::from_keyevent(&key);

        app::push_key_buf(key);
    }

    let registerd = config::KeyConfig::registerd();

    if !registerd.iter().any(|(_, map)| app::is_similar_buf(map)) {
        app::clear_key_buf();

        return Ok(false);
    }

    for (name, map) in registerd.into_iter() {
        if app::eq_buf(map) {
            name.run()?;
            app::clear_key_buf();
        }
    }

    Ok(false)
}

pub fn handle_input_mode(input: &mut Input, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            input.disable();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'h' => input.cursor_left(),
                    'l' => input.cursor_right(),
                    _ => {}
                }

                return;
            }

            input.buffer_insert(c);

            if let Some(act) = input.load_action() {
                match act.as_str() {
                    "Search" => app::sync_grep(input),
                    "RmSelected" | "RmFileOrDirectory" if config::load().rm.no_enter => {
                        handle_input_mode(input, KeyEvent::from(KeyCode::Enter));
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Delete => {
            input.buffer_pick_next();

            if input.load_action().as_deref() == Some("Search") {
                app::sync_grep(input);
            }
        }
        KeyCode::Backspace => {
            input.buffer_pick();

            if input.load_action().as_deref() == Some("Search") {
                app::sync_grep(input);
            }
        }
        KeyCode::Enter => {
            input.complete_input();

            let content = input.drain_storage();
            let act = input.drain_action();

            tokio::task::spawn_blocking(|| {
                let Some(content) = content else { return };

                if let Some(action) = act {
                    app::proc_count_up();
                    handle_action(content.trim(), action);
                    app::proc_count_down();
                }
            });
        }
        _ => {}
    }
}

fn act_add_file_or_dir(content: &str) {
    let path = app::get_path().join(content);

    if path.exists() {
        crate::sys_log!(
            "w",
            "Command AddNewFileOrDirectory failed: \"{}\" is already exists",
            content
        );
        crate::log!("Add new file failed: \"{}\" is already exists", &content);

        return;
    }

    let add_res = if content.ends_with("/") {
        std::fs::create_dir(&path)
    } else {
        std::fs::write(&path, "")
    };

    if let Err(e) = add_res {
        crate::sys_log!("w", "Command AddNewFileOrDirectory failed: {}", e.kind());
        crate::log!("Add new file failed: {}", e.kind());

        return;
    }

    cursor::load().resize(misc::child_files_len(&app::get_path()));
    crate::sys_log!(
        "w",
        "Command AddNewFileOrDirectory successful: create the {}",
        &content
    );
    crate::log!("\"{}\" create successful", &content)
}

fn act_rm_file_or_dir(content: &str, native: bool) {
    if !["y", "Y", config::load().key.delete.to_string().as_str()].contains(&content) {
        return;
    }

    if let Some(under_cursor_file) =
        misc::sorted_child_files(&app::get_path()).get(cursor::load().current())
    {
        let Ok(metadata) = under_cursor_file.symlink_metadata() else {
            crate::sys_log!(
                "w",
                "Command RmFileOrDirectory failed: target metadata cannot access"
            );
            crate::log!("Delete file failed: cannot access metadata");

            return;
        };

        if !under_cursor_file.exists() && !metadata.is_symlink() {
            crate::sys_log!(
                "w",
                "Command RmFileOrDirectory failed: target file not exists"
            );
            crate::log!("Delete file failed: target not exists");

            return;
        }

        let name = misc::file_name(under_cursor_file);

        let res = if config::load().rm.for_tmp {
            if config::load().rm.yank {
                if native && !clipboard::is_cmd_installed() {
                    crate::sys_log!(
                        "w",
                        "File yank failed: native command not installed, and config the native-clip is enabled"
                    );
                    crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

                    return;
                }

                let tmp_path = Path::new("/tmp")
                    .join("endolphine")
                    .join(misc::file_name(under_cursor_file));

                if native {
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
            crate::sys_log!("w", "Command RmFileOrDirectory failed: {}", e.kind());
            crate::log!("Delete file failed: {}", e.kind());

            return;
        }

        cursor::load().resize(misc::child_files_len(&app::get_path()));
        crate::sys_log!(
            "i",
            "Command RmFileOrDirectory successful: delete the \"{}\"",
            name
        );
        crate::log!("\"{}\" delete successful", name);
    } else {
        crate::sys_log!(
            "w",
            "Command RmFileOrDirectory failed: cursor in invalid position"
        );
        crate::log!("Delete file failed: target cannot find");
    }
}

fn act_rm_selected(content: &str, native: bool) {
    if !["y", "Y", config::load().key.delete.to_string().as_str()].contains(&content) {
        return;
    }

    let selected = misc::sorted_child_files(&app::get_path())
        .into_iter()
        .enumerate()
        .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
        .collect::<Vec<_>>();

    if config::load().rm.for_tmp {
        if config::load().rm.yank {
            if native && !clipboard::is_cmd_installed() {
                crate::sys_log!(
                    "w",
                    "File yank failed: native command not installed, and config the native-clip is enabled"
                );
                crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

                return;
            }

            let tmp = Path::new("/tmp").join("endolphine");

            use std::fmt::Write;
            let text = selected.iter().fold(String::new(), |mut acc, p| {
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

        if let Err(e) = misc::into_tmp(&selected) {
            crate::sys_log!("w", "Command RmSelected failed: {}", e.kind());
            crate::log!("Delete file failed: {}", e.kind());

            return;
        }
    } else {
        for target in &selected {
            let Ok(metadata) = target.symlink_metadata() else {
                crate::sys_log!(
                    "w",
                    "Command RmSelected failed: target metadata cannot access"
                );
                crate::log!("Delete file failed: cannot access metadata");

                return;
            };

            if !target.exists() && !metadata.is_symlink() {
                crate::sys_log!("w", "Command RmSelected failed: target file not exists");
                crate::log!("Delete file failed: target not exists");

                return;
            }

            let res = if target.is_dir() {
                misc::remove_dir_all(target)
            } else {
                std::fs::remove_file(target)
            };

            if let Err(e) = res {
                crate::sys_log!("w", "Command RmSelected failed: {}", e.kind());
                crate::log!("Delete file failed: {}", e.kind());

                return;
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
}

fn act_rename(content: &str) {
    let path = app::get_path();

    if let Some(under_cursor_file) = misc::sorted_child_files(&path).get(cursor::load().current()) {
        let renamed = path.join(content);

        let Ok(metadata) = under_cursor_file.symlink_metadata() else {
            crate::sys_log!("w", "Command Rename failed: target metadata cannot access");
            crate::log!("Rename failed: cannot access metadata");

            return;
        };

        if !under_cursor_file.exists() && !metadata.is_symlink() {
            crate::sys_log!("w", "Command Rename failed: target file not exists");
            crate::log!("Rename failed: \"{}\" is not exists", &content);

            return;
        }

        if let Err(e) = std::fs::rename(under_cursor_file, &renamed) {
            crate::sys_log!("w", "Command Rename failed: {}", e.kind());
            crate::log!("Rename failed: {}", e.kind());

            return;
        }

        crate::sys_log!(
            "i",
            "Command Rename successful: \"{}\" into the \"{}\"",
            under_cursor_file.to_string_lossy(),
            renamed.to_string_lossy()
        );
        crate::log!(
            "\"{}\" renamed to \"{}\"",
            misc::file_name(under_cursor_file),
            misc::file_name(&renamed)
        );
    }
}

fn act_paste(content: &str, native: bool) {
    let files = if native {
        match clipboard::read_clipboard_native("text/uri-list") {
            Ok(text) => text
                .lines()
                .filter_map(|f| f.strip_prefix("file://"))
                .map(PathBuf::from)
                .filter(|f| misc::exists_item(f))
                .collect::<Vec<PathBuf>>(),
            Err(e) => {
                crate::sys_log!("w", "Couldn't read a clipboard: {}", e.kind());
                crate::log!("Paste failed: {}", e.kind());

                return;
            }
        }
    } else {
        clipboard::read_clipboard()
            .split('\n')
            .map(PathBuf::from)
            .filter(|c| misc::exists_item(c))
            .collect::<Vec<_>>()
    };

    let current_path = app::get_path();
    let overwrite_mode =
        ["y", "Y", config::load().key.paste.to_string().as_str()].contains(&content);

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
}

fn act_search() {
    let cursor = cursor::load();

    let child_files = misc::sorted_child_files(&app::get_path());
    let first_match_pos = child_files[cursor.current() + 1..]
        .iter()
        .chain(child_files[..cursor.current()].iter())
        .position(|f| app::regex_match(misc::file_name(f)))
        .map(|pos| pos + 1)
        .unwrap_or(0);

    cursor.shift_loop(first_match_pos as isize);
    crate::log!("/{}", app::read_grep());
}

fn handle_action(content: &str, act: String) {
    match act.as_str() {
        "AddNewFileOrDirectory" => act_add_file_or_dir(content),
        "RmFileOrDirectory" => act_rm_file_or_dir(content, config::load().native_clip),
        "RmSelected" => act_rm_selected(content, config::load().native_clip),
        "Rename" => act_rename(content),
        "Paste" => act_paste(content, config::load().native_clip),
        "Search" => act_search(),
        _ => {}
    }
}
