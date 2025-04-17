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
        self.cursor.shift_p(initial.len());
        crate::app::switch_mode(crate::app::AppMode::Input);
    }

    pub fn disable(&mut self) {
        self.buffer = None;
        crate::app::switch_mode(crate::app::AppMode::Normal);
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

    pub fn buffer_len(&self) -> usize {
        self.buffer.as_ref().map(|b| b.len()).unwrap_or(0)
    }

    pub fn complete_input(&mut self) {
        self.storage = self.buffer.take();
        self.cursor = Cursor::default();
        crate::app::switch_mode(crate::app::AppMode::Normal);
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

fn get_ref() -> std::sync::RwLockReadGuard<'static, Input> {
    INPUT.read().unwrap()
}

pub fn is_enable() -> bool {
    get_ref().is_enable()
}

pub fn buffer() -> Option<String> {
    get_ref().buffer_load().clone()
}

pub fn buffer_len() -> usize {
    get_ref().buffer_len()
}

pub fn cursor_pos() -> usize {
    get_ref().cursor_current()
}

fn get_mut() -> std::sync::RwLockWriteGuard<'static, Input> {
    INPUT.write().unwrap()
}

pub fn enable(initial: &str, action: Option<String>) {
    get_mut().enable(initial, action);
}

pub fn use_f_mut<F: FnOnce(&mut Input) -> R, R>(f: F) -> R {
    let mut lock = INPUT.write().unwrap();
    f(&mut lock)
}
