use crate::global;
use std::sync::RwLock;

global!(INPUT<RwLock<Input>>, || RwLock::new(Input::default()), {
    fn input() -> &'static RwLock<Input> {
        &INPUT
    }
});

pub fn use_f<F: FnOnce(&Input) -> R, R>(f: F) -> R {
    let lock = input().read().unwrap();
    f(&lock)
}

pub fn use_f_mut<F: FnOnce(&mut Input) -> R, R>(f: F) -> R {
    let mut lock = input().write().unwrap();
    f(&mut lock)
}

#[derive(Default)]
pub struct Input {
    buffer: Option<String>,
    storage: Option<String>,
    action: Option<String>,
}

impl Input {
    pub fn is_enable(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn enable(&mut self, initial: &str, action: Option<String>) {
        self.buffer = Some(String::from(initial));
        self.action = action;
    }

    pub fn disable(&mut self) {
        self.buffer = None;
    }

    pub fn buffer_push(&mut self, c: char) {
        if let Some(ref mut buf) = self.buffer {
            buf.push(c);
        }
    }

    pub fn buffer_pop(&mut self) {
        if let Some(ref mut buf) = self.buffer {
            buf.pop();
        }
    }

    pub fn buffer_load(&self) -> &Option<String> {
        &self.buffer
    }

    pub fn complete_input(&mut self) {
        self.storage = self.buffer.take();
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
