pub struct Input {
    buffer: Option<String>,
    storage: Option<String>,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            buffer: None,
            storage: None,
        }
    }
}

impl Input {
    pub fn is_enable(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn enable(&mut self, initial: &str) {
        self.buffer = Some(String::from(initial));
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

    pub fn buffer_load<'a>(&'a self) -> &'a Option<String> {
        &self.buffer
    }

    pub fn complete_input(&mut self) {
        self.storage = self.buffer.clone();
        self.disable();
    }

    pub fn drain_storage(&mut self) -> Option<String> {
        let tmp = self.storage.clone();
        self.storage = None;

        tmp
    }
}
