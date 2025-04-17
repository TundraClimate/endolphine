use crate::{
    app, canvas, config, cursor,
    input::{self, Input},
    misc,
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

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
    if input::is_enable() {
        input::use_f_mut(|i| handle_input_mode(i, key));

        return Ok(());
    }

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
                if act.as_str() == "Search" {
                    app::sync_grep(input)
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

fn handle_action(content: &str, act: String) {
    use crate::command::{self, Command};
    if let Err(e) = match act.as_str() {
        "CreateFileOrDir" => command::CreateFileOrDir {
            content: content.to_string(),
            is_file: !content.ends_with("/"),
        }
        .run(),
        "DeleteFileOrDir" if content.eq_ignore_ascii_case("y") => command::DeleteFileOrDir {
            yank_and_native: (config::load().delete.yank, config::load().native_clip),
            use_tmp: config::load().delete.for_tmp,
        }
        .run(),
        "DeleteSelected" if content.eq_ignore_ascii_case("y") => command::DeleteSelected {
            yank_and_native: (config::load().delete.yank, config::load().native_clip),
            use_tmp: config::load().delete.for_tmp,
        }
        .run(),
        "Rename" => command::Rename {
            content: content.to_string(),
        }
        .run(),
        "Paste" => command::Paste {
            overwrite: content.eq_ignore_ascii_case("y"),
            native: config::load().native_clip,
        }
        .run(),
        "Search" => command::SearchNext.run(),
        _ => Ok(()),
    } {
        e.handle()
    }
}
