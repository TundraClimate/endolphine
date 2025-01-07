use std::sync::RwLock;

pub struct Input {
    buffer: RwLock<Option<String>>,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            buffer: RwLock::new(None),
        }
    }
}

impl Input {
    pub fn is_enable(&self) -> bool {
        self.buffer.read().unwrap().is_some()
    }

    pub fn toggle_enable(&self) {
        let mut lock = self.buffer.write().unwrap();
        if lock.is_some() {
            *lock = None;
        } else {
            *lock = Some(String::new());
        }
    }

    pub fn buffer_push(&self, c: char) {
        let mut lock = self.buffer.write().unwrap();
        if let Some(ref mut buf) = *lock {
            buf.push(c);
        }
    }

    pub fn buffer_pop(&self) {
        let mut lock = self.buffer.write().unwrap();
        if let Some(ref mut buf) = *lock {
            buf.pop();
        }
    }

    pub fn drain(&self) -> Option<String> {
        let mut lock = self.buffer.write().unwrap();
        let Some(ref mut buf) = *lock else {
            return None;
        };

        let inner = buf.clone();

        *lock = None;

        Some(inner)
    }
}
