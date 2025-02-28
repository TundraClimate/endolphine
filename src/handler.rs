use crate::{
    app, canvas, clipboard, config, cursor,
    error::*,
    input::{self, Input},
    menu, misc,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::path::{Path, PathBuf};

pub fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return handle_key_event(key),
            Event::Resize(_, _) => {
                cursor::master().resize(misc::child_files_len(&app::get_path()));
                canvas::cache_clear();
            }
            _ => {}
        }
    }

    Ok(false)
}

fn handle_key_event(key: KeyEvent) -> EpResult<bool> {
    if input::use_f(|i| i.is_enable()) {
        input::use_f_mut(|i| handle_input_mode(i, key));
        return Ok(false);
    }

    match key.code {
        KeyCode::Char(c) => return handle_char_key(c),
        KeyCode::Esc => handle_esc_key()?,
        _ => {}
    }

    Ok(false)
}

fn handle_input_mode(input: &mut Input, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            input.disable();
            canvas::cache_clear();
        }
        KeyCode::Char(c) => {
            input.buffer_push(c);
            if let Some(act) = input.load_action() {
                match act.as_str() {
                    "Search" => app::grep_update(|m| m.push(c)),
                    "RmSelected" | "RmFileOrDirectory" if config::load().rm.no_enter => {
                        handle_input_mode(input, KeyEvent::from(KeyCode::Enter));
                    }
                    _ => {}
                }
            }
        }
        KeyCode::Delete | KeyCode::Backspace => {
            input.buffer_pop();
            if input.load_action() == &Some("Search".to_owned()) {
                app::grep_update(|m| {
                    m.pop();
                });
            }
        }
        KeyCode::Enter => {
            input.complete_input();
            canvas::cache_clear();

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

fn handle_action(content: &str, act: String) {
    match act.as_str() {
        "AddNewFileOrDirectory" => {
            let path = app::get_path().join(content);

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

            cursor::master().resize(misc::child_files_len(&app::get_path()));
            crate::log!(format!("\"{}\" create successful.", &content))
        }
        "RmFileOrDirectory" => {
            if !["y", "Y", config::load().key.delete.to_string().as_str()].contains(&content) {
                return;
            }

            if let Some(under_cursor_file) =
                misc::sorted_child_files(&app::get_path()).get(cursor::master().current())
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

                let res = if config::load().rm.for_tmp {
                    if config::load().rm.yank {
                        if !clipboard::is_cmd_installed() {
                            crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");
                            return;
                        }

                        let tmp_path = Path::new("/tmp")
                            .join("endolphine")
                            .join(misc::file_name(under_cursor_file));

                        let text = format!("file://{}", tmp_path.to_string_lossy());
                        if let Err(e) = clipboard::clip(&text, "text/uri-list") {
                            crate::log!(format!("Yank failed: {}", e.kind()));
                        }
                    }
                    misc::into_tmp(&[under_cursor_file.to_path_buf()])
                } else if under_cursor_file.is_dir() {
                    misc::remove_dir_all(under_cursor_file)
                } else {
                    std::fs::remove_file(under_cursor_file)
                };

                if let Err(e) = res {
                    crate::log!(format!("Delete file failed: {}", e.kind()));
                    return;
                }

                cursor::master().resize(misc::child_files_len(&app::get_path()));
                crate::log!(format!("\"{}\" delete successful.", name));
            } else {
                crate::log!("Delete file failed: target cannot find.");
            }
        }
        "RmSelected" => {
            if !["y", "Y", config::load().key.delete.to_string().as_str()].contains(&content) {
                return;
            }

            let cursor = cursor::master();

            let selected = misc::sorted_child_files(&app::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .collect::<Vec<_>>();

            if config::load().rm.for_tmp {
                if config::load().rm.yank {
                    if !clipboard::is_cmd_installed() {
                        crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");
                        return;
                    }

                    let tmp = Path::new("/tmp").join("endolphine");

                    use std::fmt::Write;
                    let text = selected.iter().fold(String::new(), |mut acc, p| {
                        let _ = writeln!(
                            acc,
                            "file://{}",
                            tmp.join(misc::file_name(p)).to_string_lossy()
                        );
                        acc
                    });

                    if let Err(e) = clipboard::clip(&text, "text/uri-list") {
                        crate::log!(format!("Yank failed: {}", e.kind()));
                    }
                }
                if let Err(e) = misc::into_tmp(&selected) {
                    crate::log!(format!("Delete file failed: {}", e.kind()));
                    return;
                }
            } else {
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
                        misc::remove_dir_all(target)
                    } else {
                        std::fs::remove_file(target)
                    };

                    if let Err(e) = res {
                        crate::log!(format!("Delete file failed: {}", e.kind()));
                        return;
                    }
                }
            }

            cursor::master().resize(misc::child_files_len(&app::get_path()));
            cursor::master().disable_selection_mode();
            crate::log!(format!("{} items delete successful.", selected.len()));
        }
        "Rename" => {
            let path = app::get_path();
            if let Some(under_cursor_file) =
                misc::sorted_child_files(&path).get(cursor::master().current())
            {
                let renamed = path.join(content);

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
                    .filter_map(|f| f.strip_prefix("file://"))
                    .map(PathBuf::from)
                    .filter(|f| misc::exists_item(f))
                    .collect::<Vec<PathBuf>>(),
                Err(e) => {
                    crate::log!(format!("Paste failed: {}", e.kind()));
                    return;
                }
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
                        let added_suffix = if let Some(extension) =
                            copied.extension().map(|e| e.to_string_lossy())
                        {
                            format!("{}{}.{}", stem, suffix, extension)
                        } else {
                            format!("{}{}", stem, suffix)
                        };

                        current_path.join(PathBuf::from(added_suffix))
                    } else {
                        copied
                    }
                };

                if (metadata.is_file() || metadata.is_symlink())
                    && (!misc::exists_item(&copied_path) || overwrite_mode)
                {
                    if let Err(e) = std::fs::copy(file, &copied_path) {
                        crate::log!(format!("Paste failed: \"{}\"", e.kind()));
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
                                    crate::log!(format!("Paste failed: \"{}\"", e.kind()));
                                    continue;
                                }
                            }

                            if let Err(e) = std::fs::copy(entry.path(), &copied_path) {
                                crate::log!(format!("Paste failed: \"{}\"", e.kind()));
                            }
                        }
                    }
                }
            }
            cursor::master().resize(misc::child_files_len(&app::get_path()));

            crate::log!(format!("{} files paste successful.", files.len()));
        }
        "Search" => {
            let cursor = cursor::master();

            let child_files = misc::sorted_child_files(&app::get_path());
            let first_match_pos = child_files[cursor.current() + 1..]
                .iter()
                .chain(child_files[..cursor.current()].iter())
                .position(|f| app::is_match_grep(|m| misc::file_name(f).contains(m)))
                .map(|pos| pos + 1)
                .unwrap_or(0);

            cursor.shift_loop(first_match_pos as isize);
            crate::log!(format!("/{}", app::read_grep()));
        }
        _ => {}
    }
}

fn handle_esc_key() -> EpResult<()> {
    cursor::master().disable_selection_mode();

    Ok(())
}

fn move_current_dir(path: &Path) {
    let cursor = cursor::master();
    cursor.disable_selection_mode();
    app::set_path(path);
    canvas::cache_clear();
    app::grep_update(|m| m.clear());

    cursor.resize(misc::child_files_len(path));
    cursor.reset();
}

fn handle_char_key(key: char) -> EpResult<bool> {
    let keyconf = &config::load().key;

    if key == keyconf.exit_app {
        return Ok(true);
    }

    if [
        keyconf.move_up,
        keyconf.move_up_ten,
        keyconf.move_down,
        keyconf.move_down_ten,
    ]
    .contains(&key)
    {
        let cursor = cursor::captured();

        match key {
            c if c == keyconf.move_up => cursor.previous(),
            c if c == keyconf.move_up_ten => cursor.shift(-10),
            c if c == keyconf.move_down => cursor.next(),
            c if c == keyconf.move_down_ten => cursor.shift(10),
            _ => unreachable!(),
        };
    }

    if key == keyconf.move_parent {
        let menu = menu::refs();
        if menu.is_enabled() {
            return Ok(false);
        }

        let path = app::get_path();

        if path == PathBuf::from("/") {
            return Ok(false);
        }

        let parent = misc::parent(&path);

        let cursor = cursor::master();
        let child_files = misc::sorted_child_files(&path);
        {
            if let Some(target_path) = child_files.get(cursor.current()) {
                let mut cur = cursor.cache.write().unwrap();
                cur.wrap_node(target_path);
            }
        }

        move_current_dir(&parent);

        let child_files = misc::sorted_child_files(&parent);
        if let Some(pos) = child_files.into_iter().position(|p| p == path) {
            cursor.shift(pos as isize);
        }
    }

    if key == keyconf.enter_dir_or_edit {
        let cursor = cursor::captured();

        let menu = menu::refs();
        if menu.is_enabled() {
            let elements = menu.elements();
            if let Some(element) = elements.get(cursor.current()) {
                let path = element.path();

                if !path.is_dir() {
                    crate::log!(format!("\"{}\" is not Directory", element.tag()));
                    return Ok(false);
                }

                move_current_dir(path);
                menu.toggle_enable();

                cursor::master().cache.write().unwrap().reset();
            }

            return Ok(false);
        }

        let path = app::get_path();
        let child_files = misc::sorted_child_files(&path);

        if child_files.is_empty() {
            return Ok(false);
        }

        let Some(target_path) = child_files.get(cursor.current()) else {
            return Ok(false);
        };

        if target_path.is_dir() {
            let child_files = misc::sorted_child_files(target_path);

            move_current_dir(target_path);

            let mut cache = cursor.cache.write().unwrap();
            if let Some(pos) = child_files.iter().position(|e| cache.inner_equal(e)) {
                cursor.shift(pos as isize);
                cache.unwrap_surface();
            } else {
                cache.reset();
            }
        } else if target_path.is_file() {
            let Some(mut editor) = config::load().editor_command() else {
                crate::log!("invalid config: editor");
                return Ok(false);
            };

            crate::disable_tui!()?;
            let cmd_result = editor.arg(target_path).status().map_err(|e| {
                EpError::CommandExecute(
                    editor.get_program().to_string_lossy().into_owned(),
                    e.kind().to_string(),
                )
            });
            crate::enable_tui!()?;

            if let Err(e) = cmd_result {
                e.handle();
            }

            canvas::cache_clear();
        }
    }

    if key == keyconf.visual_select {
        cursor::master().toggle_selection();
    }

    if key == keyconf.menu_toggle {
        if !menu::is_opened() || menu::refs().is_enabled() {
            menu::refs().toggle_enable();
        }

        menu::toggle_open();
        canvas::cache_clear();
    }

    if key == keyconf.menu_move {
        if !menu::is_opened() {
            menu::toggle_open();
        }

        menu::refs().toggle_enable();
        canvas::cache_clear();
    }

    if key == keyconf.create_new {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        cursor::master().disable_selection_mode();

        input::use_f_mut(|i| i.enable("", Some("AddNewFileOrDirectory".into())));
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");
    }

    if key == keyconf.delete {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        let cursor = cursor::master();

        if cursor.is_selection_mode() {
            let selected_files = misc::sorted_child_files(&app::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .collect::<Vec<_>>();
            input::use_f_mut(|i| i.enable("", Some("RmSelected".into())));
            crate::log!(format!(
                "Delete {} items ? (y/Y/{})",
                keyconf.delete,
                selected_files.len()
            ));
            return Ok(false);
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            input::use_f_mut(|i| i.enable("", Some("RmFileOrDirectory".into())));
            crate::log!(format!(
                "Delete \"{}\" ? (y/Y/{})",
                keyconf.delete,
                misc::file_name(under_cursor_file)
            ));
        }
    }

    if key == keyconf.rename {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        let cursor = cursor::master();

        cursor.disable_selection_mode();

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            let name = misc::file_name(under_cursor_file);
            input::use_f_mut(|i| i.enable(name, Some("Rename".into())));
            crate::log!(format!("Enter new name for \"{}\"", name));
        }
    }

    if key == keyconf.yank {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        if !clipboard::is_cmd_installed() {
            crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");
            return Ok(false);
        }

        let cursor = cursor::master();

        if cursor.is_selection_mode() {
            let selected_files = misc::sorted_child_files(&app::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor.is_selected(i).then_some(f))
                .map(|f| format!("file://{}", f.to_string_lossy()))
                .collect::<Vec<_>>();

            if let Err(e) = clipboard::clip(&selected_files.join("\n"), "text/uri-list") {
                crate::log!(format!("Yank failed: {}", e.kind()));
                return Ok(false);
            }

            cursor.disable_selection_mode();
            crate::log!(format!("Yanked {} items", selected_files.len()));
            return Ok(false);
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            let text = format!("file://{}", under_cursor_file.to_string_lossy());

            if let Err(e) = clipboard::clip(&text, "text/uri-list") {
                crate::log!(format!("Yank failed: {}", e.kind()));
                return Ok(false);
            }

            crate::log!(format!("Yanked \"{}\"", misc::file_name(under_cursor_file)));
        }
    }

    if key == keyconf.paste {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        if !clipboard::is_cmd_installed() {
            crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");
            return Ok(false);
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
                handle_input_mode(i, KeyEvent::from(KeyCode::Enter));
            } else {
                crate::log!(format!("Is overwrite paste? (y/Y/{})", keyconf.paste));
            };

            Ok::<(), EpError>(())
        })?;
    }

    if [keyconf.search, keyconf.search_next].contains(&key) {
        if menu::refs().is_enabled() {
            return Ok(false);
        }

        cursor::master().disable_selection_mode();

        match key {
            c if c == keyconf.search => {
                app::grep_update(|m| m.clear());
                input::use_f_mut(|i| i.enable("/", Some("Search".to_string())));
            }
            c if c == keyconf.search_next => {
                if !app::is_match_grep(|m| m.is_empty()) {
                    input::use_f_mut(|i| {
                        i.enable("/", Some("Search".to_string()));
                        handle_input_mode(i, KeyEvent::from(KeyCode::Enter))
                    });
                }
            }
            _ => unreachable!(),
        }
    }

    Ok(false)
}
