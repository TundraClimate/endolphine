use crate::{app, canvas_cache, error::*, misc};
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use std::{path::PathBuf, process::Command};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => {
                return Ok(handle_key_event(key).await? == HandledKeyEventState::Leave)
            }
            Event::Resize(_, row) => {
                app::set_row(row);
                app::cursor().resize(misc::child_files(&app::get_path()).len());
                canvas_cache::clear();
            }
            _ => {}
        }
    }

    Ok(false)
}

#[derive(PartialEq, Eq)]
enum HandledKeyEventState {
    Leave,
    Retake,
}

async fn handle_key_event(key: KeyEvent) -> EpResult<HandledKeyEventState> {
    match key.code {
        KeyCode::Char(c) => return handle_char_key(c).await,
        _ => {}
    }
    Ok(HandledKeyEventState::Retake)
}

async fn handle_char_key(key: char) -> EpResult<HandledKeyEventState> {
    if key == 'Q' {
        return Ok(HandledKeyEventState::Leave);
    }

    if ['j', 'k', 'J', 'K'].contains(&key) {
        let cursor = app::cursor();
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
            return Ok(HandledKeyEventState::Retake);
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
        let cursor = app::cursor();
        if cursor.is_selection_mode() {
            cursor.toggle_selection();
        }

        let path = app::get_path();
        let child_files = misc::sorted_child_files(&path);

        if child_files.len() == 0 {
            return Ok(HandledKeyEventState::Retake);
        }

        let Some(target_path) = child_files.get(cursor.current()) else {
            return Ok(HandledKeyEventState::Retake);
        };
        let Ok(metadata) = target_path.symlink_metadata() else {
            return Ok(HandledKeyEventState::Retake);
        };

        match metadata {
            metadata if metadata.is_dir() => {
                let child_files = misc::sorted_child_files(target_path);

                app::set_path(target_path);
                cursor.reset();
                cursor.resize(child_files.len());

                {
                    let mut cur = cursor.cache.write().unwrap();
                    if let Some(pos) = child_files.iter().position(|e| cur.inner_equal(e)) {
                        cursor.shift(pos as isize);
                        cur.unwrap_surface();
                    } else {
                        cur.reset();
                    }
                }
            }
            metadata if metadata.is_file() => {
                let editor = option_env!("EDITOR").unwrap_or("vi");
                crate::disable_tui!().map_err(|_| EpError::SwitchScreen)?;
                Command::new(editor)
                    .arg(&target_path)
                    .status()
                    .map_err(|e| {
                        EpError::CommandExecute(editor.to_string(), e.kind().to_string())
                    })?;
                crate::enable_tui!().map_err(|_| EpError::SwitchScreen)?;
                canvas_cache::clear();
            }
            _ => {}
        }
    }

    if key == 'V' {
        app::cursor().toggle_selection();
    }

    if key == 'M' {
        if app::get_view_shift() == 0 {
            app::set_view_shift(20);
        } else {
            app::set_view_shift(0);
        }
        canvas_cache::clear();
    }

    Ok(HandledKeyEventState::Retake)
}
