use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        RwLock,
    },
};

pub struct Cursor {
    index: AtomicUsize,
    size: AtomicUsize,
    pub cache: RwLock<CursorCache>,
    selection: RwLock<Option<(usize, usize)>>,
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            index: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
            cache: RwLock::new(CursorCache::new()),
            selection: RwLock::new(None),
        }
    }

    pub fn resize(&self, new_size: usize) {
        self.size.swap(new_size, Ordering::Relaxed);
        self.keep_range(new_size);
    }

    fn keep_range(&self, size: usize) {
        self.swap_id(self.current().min(size - 1))
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

        if self.is_selection_mode() {
            let mut lock = self.selection.write().unwrap();
            if let Some((base, _)) = *lock {
                *lock = Some((base, val));
            }
        }
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

    pub fn shift_loop(&self, val: isize) {
        if val.is_positive() {
            if val as usize + self.current() < self.size.load(Ordering::Relaxed) {
                self.shift(val);
            } else {
                self.swap_id((self.current() + val as usize) - self.size.load(Ordering::Relaxed));
            }
        } else {
            self.shift(val);
        }
    }

    pub fn reset(&self) {
        self.swap_id(0);
    }

    pub fn is_selection_mode(&self) -> bool {
        self.selection.read().unwrap().is_some()
    }

    pub fn toggle_selection(&self) {
        let mut lock = self.selection.write().unwrap();
        if lock.is_some() {
            *lock = None;
        } else {
            *lock = Some((self.current(), self.current()))
        }
    }

    pub fn is_selected(&self, i: usize) -> bool {
        if !self.is_selection_mode() {
            return false;
        }

        let lock = self.selection.read().unwrap();
        if let Some((base, pin)) = *lock {
            let min = base.min(pin);
            let max = base.max(pin);
            (min..=max).contains(&i)
        } else {
            false
        }
    }
}

#[derive(Clone, Debug)]
struct CacheNode {
    data_path: PathBuf,
    inner: Option<Box<CacheNode>>,
}

#[derive(Debug)]
pub struct CursorCache {
    inner: Option<Box<CacheNode>>,
}

impl CursorCache {
    fn new() -> Self {
        CursorCache { inner: None }
    }

    pub fn wrap_node(&mut self, data_path: &PathBuf) {
        if let Some(inner) = &self.inner {
            self.inner = Some(Box::new(CacheNode {
                data_path: data_path.clone(),
                inner: Some(Box::new(*inner.clone())),
            }));
        } else {
            self.inner = Some(Box::new(CacheNode {
                data_path: data_path.clone(),
                inner: None,
            }))
        }
    }

    pub fn unwrap_surface(&mut self) {
        if let Some(inner) = &self.inner {
            self.inner = inner.inner.clone();
        }
    }

    pub fn reset(&mut self) {
        self.inner = None;
    }

    pub fn inner_equal(&self, data_path: &PathBuf) -> bool {
        if let Some(inner) = &self.inner {
            &inner.data_path == data_path
        } else {
            false
        }
    }
}
