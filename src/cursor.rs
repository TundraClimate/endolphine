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
}

impl Cursor {
    pub fn new() -> Self {
        Cursor {
            index: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
            cache: RwLock::new(CursorCache::new()),
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

    pub fn reset(&self) {
        self.swap_id(0);
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
