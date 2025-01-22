use crate::{clipboard, error::*, global, input::Input, menu, misc};
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
        global::input_use_mut(|i| handle_input_mode(i, key))?;
        return Ok(false);
    }

    match key.code {
        KeyCode::Char(c) => return handle_char_key(c).await,
        KeyCode::Esc => handle_esc_key()?,
        _ => {}
    }

    Ok(false)
}

fn handle_input_mode(input: &mut Input, key: KeyEvent) -> EpResult<()> {
    match key.code {
        KeyCode::Esc => {
            input.disable();
            global::cache_clear();
        }
        KeyCode::Char(c) => {
            input.buffer_push(c);
            if input.load_action() == &Some("Search".to_owned()) {
                global::matcher_update(|m| m.push(c));
            }
        }
        KeyCode::Delete | KeyCode::Backspace => {
            input.buffer_pop();
            if input.load_action() == &Some("Search".to_owned()) {
                global::matcher_update(|m| {
                    m.pop();
                });
            }
        }
        KeyCode::Enter => {
            input.complete_input();
            global::cache_clear();

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
        "RmSelected" => {
            if !["y", "Y", "d"].contains(&content) {
                return;
            }

            let cursor = global::cursor();

            let selected = misc::sorted_child_files(&global::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .collect::<Vec<_>>();

            for target in &selected {
                let Ok(metadata) = target.symlink_metadata() else {
                    crate::log!("Delete file failed: cannot access metadata.");
                    return;
                };

                if !target.exists() && !metadata.is_symlink() {
                    crate::log!("Delete file failed: target not exists.");
                    return;
                }

                let res = if target.is_dir() {
                    std::fs::remove_dir_all(target)
                } else {
                    std::fs::remove_file(target)
                };

                if let Err(e) = res {
                    crate::log!(format!("Delete file failed: {}", e.kind()));
                    return;
                }
            }
            global::cursor().resize(misc::child_files_len(&global::get_path()));
            crate::log!(format!("{} items delete successful.", selected.len()));
        }
        "Rename" => {
            let path = global::get_path();
            if let Some(under_cursor_file) =
                misc::sorted_child_files(&path).get(global::cursor().current())
            {
                let renamed = path.join(&content);

                let Ok(metadata) = under_cursor_file.symlink_metadata() else {
                    crate::log!("Rename failed: cannot access metadata.");
                    return;
                };

                if !under_cursor_file.exists() && !metadata.is_symlink() {
                    crate::log!(format!("Rename failed: \"{}\" is not exists.", &content));
                    return;
                }

                if let Err(e) = std::fs::rename(under_cursor_file, &renamed) {
                    crate::log!(format!("Rename failed: {}", e.kind()));
                    return;
                }

                crate::log!(format!(
                    "\"{}\" renamed to \"{}\"",
                    misc::file_name(under_cursor_file),
                    misc::file_name(&renamed)
                ));
            }
        }
        "Paste" => {
            let files = match clipboard::read_clipboard("text/uri-list") {
                Ok(text) => text
                    .lines()
                    .filter_map(|f| {
                        f.starts_with("file://")
                            .then_some(f.replacen("file://", "", 1))
                    })
                    .map(PathBuf::from)
                    .filter(|f| {
                        f.symlink_metadata()
                            .ok()
                            .map_or(false, |m| m.is_symlink() || f.exists())
                    })
                    .collect::<Vec<PathBuf>>(),
                Err(e) => {
                    crate::log!(format!("Paste failed: {}", e.kind()));
                    return;
                }
            };

            let current_path = global::get_path();
            let overwrite_mode = ["y", "Y", "p"].contains(&content);

            for file in files.into_iter() {
                let Ok(metadata) = file.symlink_metadata() else {
                    continue;
                };

                if !file.exists() && !metadata.is_symlink() {
                    continue;
                }

                let copied_path = {
                    let copied = current_path.join(misc::file_name(&file));
                    if &copied == &file {
                        let stem = copied
                            .file_stem()
                            .map(|s| String::from(s.to_string_lossy()))
                            .unwrap_or(String::new());
                        current_path.join(PathBuf::from(
                            if let Some(extension) = copied.extension().map(|e| e.to_string_lossy())
                            {
                                format!("{}_Copy.{}", stem, extension)
                            } else {
                                format!("{}_Copy", stem)
                            },
                        ))
                    } else {
                        copied
                    }
                };

                if metadata.is_file() || metadata.is_symlink() {
                    if !copied_path
                        .symlink_metadata()
                        .ok()
                        .map_or(false, |m| m.is_symlink() || copied_path.exists())
                        || overwrite_mode
                    {
                        if let Err(e) = std::fs::copy(&file, &copied_path) {
                            crate::log!(format!("Paste failed: \"{}\"", e.kind()));
                        }
                    }
                }

                if metadata.is_dir() {
                    for entry in walkdir::WalkDir::new(&file)
                        .into_iter()
                        .filter_map(Result::ok)
                    {
                        let Ok(rel_path) = entry.path().strip_prefix(&file) else {
                            continue;
                        };

                        let copied_path = copied_path.join(rel_path);
                        if !copied_path
                            .symlink_metadata()
                            .ok()
                            .map_or(false, |m| m.is_symlink() || copied_path.exists())
                            || overwrite_mode
                        {
                            let parent = misc::parent(&copied_path);
                            if !parent.exists() {
                                if let Err(e) = std::fs::create_dir_all(parent) {
                                    crate::log!(format!("Paste failed: \"{}\"", e.kind()));
                                    continue;
                                }
                            }

                            if let Err(e) = std::fs::copy(&entry.path(), &copied_path) {
                                crate::log!(format!("Paste failed: \"{}\"", e.kind()));
                            }
                        }
                    }
                }
                global::cursor().resize(misc::child_files_len(&global::get_path()));
            }
        }
        "Search" => {
            misc::next_match_from_search();
            crate::log!(format!("/{}", global::read_matcher()));
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
        let menu = global::menu();
        if menu.is_enabled() {
            return Ok(false);
        }

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

        {
            let cursor = global::cursor();
            if cursor.is_selection_mode() {
                cursor.toggle_selection();
            }
        }

        global::input_use_mut(|i| i.enable("", Some("AddNewFileOrDirectory".into())));
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");
    }

    if key == 'd' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        let cursor = global::cursor();

        if cursor.is_selection_mode() {
            let selected_files = misc::sorted_child_files(&global::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .collect::<Vec<_>>();
            cursor.toggle_selection();
            global::input_use_mut(|i| i.enable("", Some("RmSelected".into())));
            crate::log!(format!("Delete {} items ? (y/Y/d)", selected_files.len()));
            return Ok(false);
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&global::get_path()).get(cursor.current())
        {
            global::input_use_mut(|i| i.enable("", Some("RmFileOrDirectory".into())));
            crate::log!(format!(
                "Delete \"{}\" ? (y/Y/d)",
                misc::file_name(under_cursor_file)
            ));
        }
    }

    if key == 'r' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        let cursor = global::cursor();

        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&global::get_path()).get(cursor.current())
        {
            let name = misc::file_name(under_cursor_file);
            global::input_use_mut(|i| i.enable(&name, Some("Rename".into())));
            crate::log!(format!("Enter new name for \"{}\"", name));
        }
    }

    if key == 'y' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        if !clipboard::is_cmd_installed() {
            crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");
            return Ok(false);
        }

        let cursor = global::cursor();

        if cursor.is_selection_mode() {
            let selected_files = misc::sorted_child_files(&global::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .map(|f| format!("file://{}", f.to_string_lossy()))
                .collect::<Vec<_>>();

            if let Err(e) = clipboard::clip(&selected_files.join("\n"), "text/uri-list") {
                crate::log!(format!("Yank failed: {}", e.kind()));
                return Ok(false);
            }

            cursor.toggle_selection();
            crate::log!(format!("Yanked {} items", selected_files.len()));
            return Ok(false);
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&global::get_path()).get(cursor.current())
        {
            let text = format!("file://{}", under_cursor_file.to_string_lossy());

            if let Err(e) = clipboard::clip(&text, "text/uri-list") {
                crate::log!(format!("Yank failed: {}", e.kind()));
                return Ok(false);
            }

            crate::log!(format!("Yanked \"{}\"", misc::file_name(under_cursor_file)));
        }
    }

    if key == 'p' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        if !clipboard::is_cmd_installed() {
            crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");
            return Ok(false);
        }

        global::input_use_mut(|i| {
            let default_paste_input = "y";

            i.enable(default_paste_input, Some("Paste".into()));

            //FIXME should impl force_mode
            if true {
                handle_input_mode(i, KeyEvent::from(KeyCode::Enter))?;
            } else {
                crate::log!("Is overwrite paste? (y/Y/p)");
            };

            Ok::<(), EpError>(())
        })?;
    }

    if key == '/' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        let cursor = global::cursor();

        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        global::matcher_update(|m| m.clear());
        global::input_use_mut(|i| i.enable("/", Some("Search".to_string())));
    }

    if key == 'n' {
        if global::menu().is_enabled() {
            return Ok(false);
        }

        let cursor = global::cursor();

        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        misc::next_match_from_search();
    }

    Ok(false)
}
