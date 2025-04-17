use crate::{app, canvas, config, cursor, misc};
use crossterm::event::{self, Event, KeyEvent};

pub fn handle_event() -> Result<(), app::Error> {
    if let Ok(event) = event::read() {
        match event {
            Event::Key(key) => handle_key_event(key)?,
            Event::Resize(_, _) => {
                cursor::load().resize(misc::child_files_len(&app::get_path()));
                canvas::cache_clear();
            }
            _ => {}
        }
    }

    Ok(())
}

fn handle_key_event(key: KeyEvent) -> Result<(), app::Error> {
    {
        let key = crate::key::Key::from_keyevent(&key);

        app::push_key_buf(key);
    }

    if !config::has_similar_map(&app::load_buf(), app::current_mode()?) {
        app::clear_key_buf();

        return Ok(());
    }

    if let Some(cmd_res) = config::eval_keymap(app::current_mode()?, &app::load_buf()) {
        app::clear_key_buf();
        cmd_res?
    }

    Ok(())
}
