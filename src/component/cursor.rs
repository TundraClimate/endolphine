use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Cursor {
    index: AtomicUsize,
    size: AtomicUsize,
}

impl Default for Cursor {
    fn default() -> Self {
        Cursor {
            index: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }
}

impl Cursor {
    pub fn resize(&self, new_size: usize) {
        self.size.swap(new_size, Ordering::Relaxed);
        self.swap_id(self.current());
    }

    pub fn current(&self) -> usize {
        self.index.load(Ordering::Relaxed)
    }

    fn swap_id(&self, val: usize) {
        let size = self.size.load(Ordering::Relaxed);

        if size == 0 {
            return;
        }

        let val = val.min(size - 1);

        self.index.swap(val, Ordering::Relaxed);
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }

    pub fn next(&self) {
        self.swap_id(self.current() + 1);
    }

    pub fn previous(&self) {
        self.swap_id(self.current().saturating_sub(1));
    }

    pub fn shift_p(&self, val: usize) {
        self.swap_id(self.current() + val);
    }

    pub fn shift_n(&self, val: usize) {
        self.swap_id(self.current().saturating_sub(val));
    }

    pub fn shift_loop_p(&self, val: usize) {
        if val + self.current() < self.size.load(Ordering::Relaxed) {
            self.shift_p(val);
        } else {
            self.swap_id((self.current() + val) - self.size.load(Ordering::Relaxed));
        }
    }

    pub fn reset(&self) {
        self.index.swap(0, Ordering::Relaxed);
    }
}
