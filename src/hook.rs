pub struct Hook {
    hook: std::sync::atomic::AtomicBool,
}

impl Default for Hook {
    fn default() -> Self {
        Self::new()
    }
}

impl Hook {
    pub fn new() -> Self {
        Self {
            hook: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn pull(&self) {
        self.hook.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn effect<F: FnOnce() -> R, R: Sized>(&self, f: F) -> Option<R> {
        if self.hook.swap(false, std::sync::atomic::Ordering::Relaxed) {
            Some(f())
        } else {
            None
        }
    }
}
