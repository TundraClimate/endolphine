use crate::{app, error::*};
use crossterm::event::{self, Event, KeyCode, KeyEvent};

pub async fn handle_event() -> EpResult<bool> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => return Ok(handle_key_event(key)? == HandledKeyEventState::Leave),
            Event::Resize(_, row) => {
                app::set_row(row);
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
        KeyCode::Char(c) => {
            if c == 'Q' {
                return Ok(HandledKeyEventState::Leave);
            }
        }
        _ => {}
    }
    Ok(HandledKeyEventState::Retake)
}
