use super::Cursor;
use std::sync::RwLock;

pub struct Input {
    input: RwLock<String>,
    cursor: Cursor,
}

impl Input {
    pub fn new() -> Self {
        Self {
            input: RwLock::new(String::new()),
            cursor: Cursor::default(),
        }
    }

    pub fn put(&self, c: char) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        input.insert(cursor.current(), c);

        cursor.resize(input.len() + 1);
        cursor.next();
    }

    pub fn pop(&self) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        if cursor.current() == 0 {
            return;
        }

        input.remove(cursor.current() - 1);

        cursor.previous();
        cursor.resize(input.len() + 1);
    }

    pub fn pop_front(&self) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        if input.len() > self.cursor.current() {
            input.remove(self.cursor.current());
        }

        cursor.resize(input.len() + 1);
    }

    pub fn insert(&self, s: &str) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        cursor.resize(input.len() + s.len());

        *input = format!(
            "{}{}{}",
            &input[..cursor.current()],
            s,
            &input[cursor.current()..]
        );

        cursor.shift_p(s.len());
    }

    pub fn shift(&self) {
        self.cursor.next();
    }

    pub fn shift_back(&self) {
        self.cursor.previous();
    }

    pub fn take(&self) -> String {
        use std::mem;

        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        cursor.resize(0);
        cursor.reset();

        mem::take(&mut input)
    }

    pub fn buf_clone(&self) -> String {
        self.input.read().unwrap().clone()
    }
}
