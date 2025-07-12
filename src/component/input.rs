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
        self.shift();
    }

    pub fn pop(&self) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        *input = input
            .char_indices()
            .flat_map(|(i, c)| (i != cursor.current()).then_some(c))
            .collect::<String>();
        cursor.resize(input.len() + 1);
        self.shift_back();
    }

    pub fn insert(&self, s: &str) {
        let mut input = self.input.write().unwrap();
        let cursor = &self.cursor;

        cursor.resize(input.len() + s.len() + 1);

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
}
