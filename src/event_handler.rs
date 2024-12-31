use crate::{app, canvas_cache, error::*, menu, misc};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{path::PathBuf, process::Command};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return Ok(handle_key_event(key).await?),
            Event::Resize(_, row) => {
                app::set_row(row);
                app::cursor().resize(misc::child_files_len(&app::get_path()));
                canvas_cache::clear();
            }
            _ => {}
        }
    }

    Ok(false)
}

async fn handle_key_event(key: KeyEvent) -> EpResult<bool> {
    match key.code {
        KeyCode::Char(c) => return handle_char_key(c).await,
        KeyCode::Esc => handle_esc_key()?,
        _ => {}
    }
    Ok(false)
}

fn handle_esc_key() -> EpResult<()> {
    let cursor = app::cursor();
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
        let cursor = app::captured_cursor();

        match key {
            'j' => cursor.next(),
            'k' => cursor.previous(),
            'J' => cursor.shift(10),
            'K' => cursor.shift(-10),
            _ => {}
        };
    }

    if key == 'h' {
        let cursor = app::cursor();
        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        let path = app::get_path();

        if path == PathBuf::from("/") {
            return Ok(false);
        }

        let parent = misc::parent(&path);
        app::set_path(&parent);
        canvas_cache::clear();

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
        let cursor = app::captured_cursor();

        let menu = app::menu();
        if menu.is_enabled() {
            let elements = menu.elements();
            if let Some(element) = elements.get(menu.cursor().current()) {
                let path = element.path();

                if !path.is_dir() {
                    crate::log!(format!("\"{}\" is not Directory", element.tag()))?;
                    return Ok(false);
                }

                app::set_path(&path);
                menu.toggle_enable();

                let cursor = app::cursor();
                cursor.reset();
                cursor.resize(misc::child_files_len(&path));
                cursor.cache.write().unwrap().reset();
            }

            return Ok(false);
        }

        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        let path = app::get_path();
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

            app::set_path(target_path);
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
            canvas_cache::clear();
        }
    }

    if key == 'V' {
        app::cursor().toggle_selection();
    }

    if key == 'M' {
        if !menu::is_opened() || app::menu().is_enabled() {
            app::menu().toggle_enable();
        }

        menu::toggle_open();
        canvas_cache::clear();
    }

    if key == 'm' {
        if menu::is_opened() {
            app::menu().toggle_enable();
            canvas_cache::clear();
        }
    }

    Ok(false)
}
