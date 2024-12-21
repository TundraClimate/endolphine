use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Cursor {
    index: AtomicUsize,
    size: AtomicUsize,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            index: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }

    pub fn resize(&self, new_size: usize) {
        self.size.swap(new_size, Ordering::Relaxed);
    }

    pub fn current(&self) -> usize {
        self.index.load(Ordering::Relaxed)
    }

    fn swap_id(&self, val: usize) {
        let size = self.size.load(Ordering::Relaxed);
        if size <= val {
            self.index
                .swap(usize::min(size - 1, val), Ordering::Relaxed);
            return;
        }

        self.index.swap(val, Ordering::Relaxed);
    }

    pub fn next(&self) {
        self.swap_id(self.current() + 1);
    }

    pub fn previous(&self) {
        self.swap_id(self.current().saturating_sub(1));
    }

    pub fn shift(&self, val: isize) {
        if val.is_positive() {
            self.swap_id(self.current() + val as usize);
        } else {
            self.swap_id(self.current().saturating_sub(-val as usize));
        }
    }
}
