use crate::{app, canvas_cache, error::*};
use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return Ok(handle_key_event(key)? == HandledKeyEventState::Leave),
            Event::Resize(_, row) => {
                app::set_row(row);
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

    if key == 'f' {
        app::set_path(std::path::PathBuf::from("/home/tundra/MyStorage"));
    }
    Ok(HandledKeyEventState::Retake)
}
