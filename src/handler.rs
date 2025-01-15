use crate::{error::*, global, menu, misc};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{path::PathBuf, process::Command};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return Ok(handle_key_event(key).await?),
            Event::Resize(width, height) => {
                global::set_width(width);
                global::set_height(height);
                global::cursor().resize(misc::child_files_len(&global::get_path()));
                global::cache_clear();
            }
            _ => {}
        }
    }

    Ok(false)
}

async fn handle_key_event(key: KeyEvent) -> EpResult<bool> {
    if global::input_use(|i| i.is_enable()) {
        handle_input_mode(key)?;
        return Ok(false);
    }

    match key.code {
        KeyCode::Char(c) => return handle_char_key(c).await,
        KeyCode::Esc => handle_esc_key()?,
        _ => {}
    }

    Ok(false)
}

fn handle_input_mode(key: KeyEvent) -> EpResult<()> {
    let mut input = global::input().write().unwrap();
    match key.code {
        KeyCode::Esc => input.disable(),
        KeyCode::Char(c) => input.buffer_push(c),
        KeyCode::Delete | KeyCode::Backspace => input.buffer_pop(),
        KeyCode::Enter => {
            input.complete_input();
            let content = input.drain_storage();
            let act = input.drain_action();
            tokio::task::spawn_blocking(|| {
                let Some(content) = content else { return };

                if let Some(action) = act {
                    handle_action(content.trim(), action);
                }
            });
        }
        _ => {}
    }

    Ok(())
}

fn handle_action(content: &str, act: String) {
    match act.as_str() {
        "AddNewFileOrDirectory" => {
            let path = global::get_path().join(&content);

            if path.exists() {
                crate::log!(format!(
                    "Add new file failed: \"{}\" is already exists.",
                    &content
                ));
                return;
            }

            if let Err(e) = if content.ends_with("/") {
                std::fs::create_dir(&path)
            } else {
                std::fs::write(&path, "")
            } {
                crate::log!(format!("Add new file failed: {}", e.kind()));
                return;
            }

            global::cursor().resize(misc::child_files_len(&global::get_path()));
            crate::log!(format!("\"{}\" create successful.", &content))
        }
        "RmFileOrDirectory" => {
            if !["y", "Y", "d"].contains(&content) {
                return;
            }

            if let Some(under_cursor_file) =
                misc::sorted_child_files(&global::get_path()).get(global::cursor().current())
            {
                let Ok(metadata) = under_cursor_file.symlink_metadata() else {
                    crate::log!("Delete file failed: cannot access metadata.");
                    return;
                };

                if !under_cursor_file.exists() && !metadata.is_symlink() {
                    crate::log!("Delete file failed: target not exists.");
                    return;
                }

                let name = misc::file_name(under_cursor_file);

                let res = if under_cursor_file.is_dir() {
                    std::fs::remove_dir_all(under_cursor_file)
                } else {
                    std::fs::remove_file(under_cursor_file)
                };

                if let Err(e) = res {
                    crate::log!(format!("Delete file failed: {}", e.kind()));
                    return;
                }

                global::cursor().resize(misc::child_files_len(&global::get_path()));
                crate::log!(format!("\"{}\" delete successful.", name));
            } else {
                crate::log!("Delete file failed: target cannot find.");
                return;
            }
        }
        _ => {}
    }
}

fn handle_esc_key() -> EpResult<()> {
    let cursor = global::cursor();
    if cursor.is_selection_mode() {
        cursor.toggle_selection();
    }

    Ok(())
}

async fn handle_char_key(key: char) -> EpResult<bool> {
    if key == 'Q' {
        return Ok(true);
    }

    if ['j', 'k', 'J', 'K'].contains(&key) {
        let cursor = global::captured_cursor();

        match key {
            'j' => cursor.next(),
            'k' => cursor.previous(),
            'J' => cursor.shift(10),
            'K' => cursor.shift(-10),
            _ => {}
        };
    }

    if key == 'h' {
        let cursor = global::cursor();
        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        let path = global::get_path();

        if path == PathBuf::from("/") {
            return Ok(false);
        }

        let parent = misc::parent(&path);
        global::set_path(&parent);
        global::cache_clear();

        let child_files = misc::sorted_child_files(&path);
        {
            if let Some(target_path) = child_files.get(cursor.current()) {
                let mut cur = cursor.cache.write().unwrap();
                cur.wrap_node(&target_path);
            }
        }
        cursor.reset();

        let child_files = misc::sorted_child_files(&parent);
        cursor.resize(child_files.len());
        if let Some(pos) = child_files.into_iter().position(|p| p == path) {
            cursor.shift(pos as isize);
        }
    }

    if key == 'l' {
        let cursor = global::captured_cursor();

        let menu = global::menu();
        if menu.is_enabled() {
            let elements = menu.elements();
            if let Some(element) = elements.get(menu.cursor().current()) {
                let path = element.path();

                if !path.is_dir() {
                    crate::log!(format!("\"{}\" is not Directory", element.tag()));
                    return Ok(false);
                }

                global::set_path(&path);
                menu.toggle_enable();
                global::cache_clear();

                let cursor = global::cursor();
                cursor.reset();
                cursor.resize(misc::child_files_len(&path));
                cursor.cache.write().unwrap().reset();
            }

            return Ok(false);
        }

        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        let path = global::get_path();
        let child_files = misc::sorted_child_files(&path);

        if child_files.len() == 0 {
            return Ok(false);
        }

        let Some(target_path) = child_files.get(cursor.current()) else {
            return Ok(false);
        };
        let Ok(metadata) = target_path.metadata() else {
            return Ok(false);
        };

        if metadata.is_dir() {
            let child_files = misc::sorted_child_files(target_path);

            global::set_path(target_path);
            cursor.reset();
            cursor.resize(child_files.len());

            let mut cur = cursor.cache.write().unwrap();
            if let Some(pos) = child_files.iter().position(|e| cur.inner_equal(e)) {
                cursor.shift(pos as isize);
                cur.unwrap_surface();
            } else {
                cur.reset();
            }
        } else if metadata.is_file() {
            let editor = option_env!("EDITOR").unwrap_or("vi");
            crate::disable_tui!()?;
            Command::new(editor)
                .arg(&target_path)
                .status()
                .map_err(|e| EpError::CommandExecute(editor.to_string(), e.kind().to_string()))?;
            crate::enable_tui!()?;
            global::cache_clear();
        }
    }

    if key == 'V' {
        global::cursor().toggle_selection();
    }

    if key == 'M' {
        if !menu::is_opened() || global::menu().is_enabled() {
            global::menu().toggle_enable();
        }

        menu::toggle_open();
        global::cache_clear();
    }

    if key == 'm' {
        if !menu::is_opened() {
            menu::toggle_open();
        }

        global::menu().toggle_enable();
        global::cache_clear();
    }

    if key == 'a' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        global::input_use_mut(|i| i.enable("", Some("AddNewFileOrDirectory".into())));
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");
    }

    if key == 'd' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&global::get_path()).get(global::cursor().current())
        {
            global::input_use_mut(|i| i.enable("", Some("RmFileOrDirectory".into())));
            crate::log!(format!(
                "Delete \"{}\" ? (y/Y/d)",
                misc::file_name(under_cursor_file)
            ));
        }
    }

    Ok(false)
}
