use crate::{cursor::Cursor, global};
use std::sync::RwLock;

#[derive(Default)]
pub struct Input {
    buffer: Option<String>,
    storage: Option<String>,
    action: Option<String>,
    cursor: Cursor,
}

impl Input {
    pub fn is_enable(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn enable(&mut self, initial: &str, action: Option<String>) {
        self.buffer = Some(String::from(initial));
        self.action = action;
        self.cursor.resize(initial.len() + 1);
        self.cursor.shift(initial.len() as isize);
    }

    pub fn disable(&mut self) {
        self.buffer = None;
    }

    pub fn buffer_insert(&mut self, c: char) {
        if let Some(ref mut buf) = self.buffer {
            buf.insert(self.cursor.current(), c);
            self.cursor.resize(buf.len() + 1);
            self.cursor.next();
        }
    }

    pub fn buffer_pick(&mut self) {
        if let Some(ref mut buf) = self.buffer {
            if self.cursor.current() != 0 {
                buf.remove(self.cursor.current() - 1);
            }
            self.cursor.previous();
            self.cursor.resize(buf.len() + 1);
        }
    }

    pub fn buffer_pick_next(&mut self) {
        if let Some(ref mut buf) = self.buffer {
            if buf.len() > self.cursor.current() {
                buf.remove(self.cursor.current());
            }
            self.cursor.resize(buf.len() + 1);
        }
    }

    pub fn cursor_left(&mut self) {
        self.cursor.previous();
    }

    pub fn cursor_right(&mut self) {
        self.cursor.next();
    }

    pub fn cursor_current(&self) -> usize {
        self.cursor.current()
    }

    pub fn buffer_load(&self) -> &Option<String> {
        &self.buffer
    }

    pub fn complete_input(&mut self) {
        self.storage = self.buffer.take();
        self.cursor = Cursor::default();
    }

    pub fn drain_storage(&mut self) -> Option<String> {
        self.storage.take()
    }

    pub fn drain_action(&mut self) -> Option<String> {
        self.action.take()
    }

    pub fn load_action(&self) -> &Option<String> {
        &self.action
    }
}

global! {
    static INPUT: RwLock<Input> = RwLock::new(Input::default());
}

pub fn use_f<F: FnOnce(&Input) -> R, R>(f: F) -> R {
    let lock = INPUT.read().unwrap();
    f(&lock)
}

pub fn use_f_mut<F: FnOnce(&mut Input) -> R, R>(f: F) -> R {
    let mut lock = INPUT.write().unwrap();
    f(&mut lock)
}
