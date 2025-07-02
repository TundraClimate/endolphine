use crate::state::State;
use crossterm::event::{self, Event, KeyEvent};
use std::sync::Arc;
use tokio::task::JoinHandle;
use viks::Key;

pub fn spawn_reader(state: Arc<State>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match event::read() {
                Ok(Event::Key(key)) => on_key(state.clone(), key),
                Ok(Event::Resize(cols, rows)) => on_resize(cols, rows),
                _ => {}
            }
        }
    })
}

fn on_key(state: Arc<State>, key: KeyEvent) {
    use crate::config;
    use viks::Keymap;

    let Some(key) = translate_to_key(key) else {
        return;
    };

    let buffer = &state.key_buffer;

    buffer.push(key);

    let keys = buffer.drain();
    let current_mode = state.mode.get();
    let keymaps = &config::get().keymaps;

    keymaps
        .eval_keys(current_mode, keys)
        .into_iter()
        .for_each(|cmd| match cmd {
            Ok((cmd, ctx)) => cmd.run(state.clone(), ctx),
            Err(mut keys) => {
                let map = Keymap::from(
                    keys.iter()
                        .skip_while(|key| key.to_string().chars().all(char::is_numeric))
                        .copied()
                        .collect::<Vec<_>>(),
                );

                if keymaps.has_similar_map(current_mode, map) {
                    buffer.append(&mut keys)
                }
            }
        });
}

fn on_resize(cols: u16, rows: u16) {}

fn translate_to_key(key: KeyEvent) -> Option<Key> {
    use crossterm::event::{KeyCode, KeyModifiers};

    let mut key_str = match key.code {
        KeyCode::Backspace => "BS",
        KeyCode::Tab => "TAB",
        KeyCode::Enter => "ENTER",
        KeyCode::Esc => "ESC",
        KeyCode::Char(' ') => "SPACE",
        KeyCode::Char('!') => "!",
        KeyCode::Char('"') => "\"",
        KeyCode::Char('#') => "#",
        KeyCode::Char('$') => "$",
        KeyCode::Char('%') => "%",
        KeyCode::Char('&') => "&",
        KeyCode::Char('\'') => "'",
        KeyCode::Char('(') => "(",
        KeyCode::Char(')') => ")",
        KeyCode::Char('*') => "*",
        KeyCode::Char('+') => "+",
        KeyCode::Char(',') => ",",
        KeyCode::Char('-') => "-",
        KeyCode::Char('.') => ".",
        KeyCode::Char('/') => "/",
        KeyCode::Char('0') => "0",
        KeyCode::Char('1') => "1",
        KeyCode::Char('2') => "2",
        KeyCode::Char('3') => "3",
        KeyCode::Char('4') => "4",
        KeyCode::Char('5') => "5",
        KeyCode::Char('6') => "6",
        KeyCode::Char('7') => "7",
        KeyCode::Char('8') => "8",
        KeyCode::Char('9') => "9",
        KeyCode::Char(':') => ":",
        KeyCode::Char(';') => ";",
        KeyCode::Char('<') => "lt",
        KeyCode::Char('=') => "=",
        KeyCode::Char('>') => ">",
        KeyCode::Char('?') => "?",
        KeyCode::Char('@') => "@",
        KeyCode::Char('a') => "a",
        KeyCode::Char('b') => "b",
        KeyCode::Char('c') => "c",
        KeyCode::Char('d') => "d",
        KeyCode::Char('e') => "e",
        KeyCode::Char('f') => "f",
        KeyCode::Char('g') => "g",
        KeyCode::Char('h') => "h",
        KeyCode::Char('i') => "i",
        KeyCode::Char('j') => "j",
        KeyCode::Char('k') => "k",
        KeyCode::Char('l') => "l",
        KeyCode::Char('m') => "m",
        KeyCode::Char('n') => "n",
        KeyCode::Char('o') => "o",
        KeyCode::Char('p') => "p",
        KeyCode::Char('q') => "q",
        KeyCode::Char('r') => "r",
        KeyCode::Char('s') => "s",
        KeyCode::Char('t') => "t",
        KeyCode::Char('u') => "u",
        KeyCode::Char('v') => "v",
        KeyCode::Char('w') => "w",
        KeyCode::Char('x') => "x",
        KeyCode::Char('y') => "y",
        KeyCode::Char('z') => "z",
        KeyCode::Char('A') => "A",
        KeyCode::Char('B') => "B",
        KeyCode::Char('C') => "C",
        KeyCode::Char('D') => "D",
        KeyCode::Char('E') => "E",
        KeyCode::Char('F') => "F",
        KeyCode::Char('G') => "G",
        KeyCode::Char('H') => "H",
        KeyCode::Char('I') => "I",
        KeyCode::Char('J') => "J",
        KeyCode::Char('K') => "K",
        KeyCode::Char('L') => "L",
        KeyCode::Char('M') => "M",
        KeyCode::Char('N') => "N",
        KeyCode::Char('O') => "O",
        KeyCode::Char('P') => "P",
        KeyCode::Char('Q') => "Q",
        KeyCode::Char('R') => "R",
        KeyCode::Char('S') => "S",
        KeyCode::Char('T') => "T",
        KeyCode::Char('U') => "U",
        KeyCode::Char('V') => "V",
        KeyCode::Char('W') => "W",
        KeyCode::Char('X') => "X",
        KeyCode::Char('Y') => "Y",
        KeyCode::Char('Z') => "Z",
        KeyCode::Char('[') => "[",
        KeyCode::Char('\\') => "\"",
        KeyCode::Char(']') => "]",
        KeyCode::Char('^') => "^",
        KeyCode::Char('_') => "_",
        KeyCode::Char('`') => "`",
        KeyCode::Char('{') => "{",
        KeyCode::Char('|') => "|",
        KeyCode::Char('}') => "}",
        KeyCode::Char('~') => "~",
        KeyCode::Delete => "DEL",
        _ => return None,
    }
    .to_string();

    let is_big_alpha = key_str.len() == 1 && matches!(key_str.chars().next(), Some('A'..='Z'));

    if !is_big_alpha && key.modifiers.contains(KeyModifiers::SHIFT) {
        key_str = format!("s-{}", key_str);
    } else if key.modifiers.contains(KeyModifiers::ALT) {
        key_str = format!("a-{}", key_str);
    } else if key.modifiers.contains(KeyModifiers::CONTROL) {
        key_str = format!("c-{}", key_str);
    }

    let key_str = if key_str.len() > 1 {
        format!("<{}>", key_str)
    } else {
        key_str
    };

    let Ok(key) = Key::new(&key_str) else {
        panic!("event translate failed: code bug");
    };

    Some(key)
}
