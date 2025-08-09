use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct Selection {
    is_enable: AtomicBool,
    start: AtomicUsize,
    end: AtomicUsize,
}

impl Selection {
    pub fn new() -> Self {
        Self {
            is_enable: AtomicBool::new(false),
            start: AtomicUsize::new(0),
            end: AtomicUsize::new(0),
        }
    }

    pub fn is_enable(&self) -> bool {
        self.is_enable.load(Ordering::Relaxed)
    }

    pub fn enable(&self, pos: usize) {
        self.is_enable.store(true, Ordering::Relaxed);
        self.start.store(pos, Ordering::Relaxed);
        self.end.store(pos, Ordering::Relaxed);
    }

    pub fn disable(&self) {
        self.is_enable.store(false, Ordering::Relaxed);
    }

    pub fn select(&self, pos: usize) {
        self.end.store(pos, Ordering::Relaxed);
    }

    pub fn collect(&self) -> Vec<usize> {
        if self.is_enable() {
            let start = self.start.load(Ordering::Relaxed);
            let end = self.end.load(Ordering::Relaxed);

            (start.min(end)..=start.max(end)).collect()
        } else {
            vec![]
        }
    }
}
