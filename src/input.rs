pub struct Input {
    buffer: Option<String>,
}

impl Default for Input {
    fn default() -> Self {
        Input { buffer: None }
    }
}

impl Input {
    pub fn is_enable(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn toggle_enable(&mut self) {
        let buffer = &mut self.buffer;
        if buffer.is_some() {
            *buffer = None;
        } else {
            *buffer = Some(String::new());
        }
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

    pub fn buffer_load(&self) -> Option<String> {
        let Some(ref buf) = self.buffer else {
            return None;
        };

        Some(buf.clone())
    }
}
