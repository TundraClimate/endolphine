use crate::{app, canvas_cache, error::*, misc};
use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return Ok(handle_key_event(key)? == HandledKeyEventState::Leave),
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

fn handle_key_event(key: KeyEvent) -> EpResult<HandledKeyEventState> {
    match key.code {
        KeyCode::Char(c) => return handle_char_key(c),
        _ => {}
    }
    Ok(HandledKeyEventState::Retake)
}

fn handle_char_key(key: char) -> EpResult<HandledKeyEventState> {
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

    Ok(HandledKeyEventState::Retake)
}
