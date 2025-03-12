use crate::global;
use std::{
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

global! {
    static CURSOR: Cursor = Cursor::new();
}

pub fn load() -> &'static Cursor {
    &CURSOR
}

pub fn captured() -> &'static Cursor {
    if crate::menu::refs().is_enabled() {
        &crate::menu::refs().cursor
    } else {
        load()
    }
}

global! {
    static SELECTION: RwLock<Option<(usize, usize)>> = RwLock::new(None);
}

pub fn is_selection() -> bool {
    SELECTION.read().is_ok_and(|i| i.is_some())
}

pub fn disable_selection() {
    *SELECTION.write().unwrap() = None;
}

pub fn toggle_selection(init: usize) {
    let mut lock = SELECTION.write().unwrap();
    if lock.is_some() {
        *lock = None;
    } else {
        *lock = Some((init, init));
    }
}

pub fn is_selected(i: usize) -> bool {
    if !is_selection() {
        return false;
    }

    let lock = SELECTION.read().unwrap();
    if let Some((base, pin)) = *lock {
        let min = base.min(pin);
        let max = base.max(pin);
        (min..=max).contains(&i)
    } else {
        false
    }
}

pub fn select_area(other: usize) {
    let mut lock = SELECTION.write().unwrap();
    if let Some((base, _)) = *lock {
        *lock = Some((base, other));
    }
}

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
        self.index.swap(0, Ordering::Relaxed);
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

    pub fn wrap_node(&mut self, data_path: &Path) {
        if let Some(inner) = &self.inner {
            self.inner = Some(Box::new(CacheNode {
                data_path: data_path.to_path_buf(),
                inner: Some(Box::new(*inner.clone())),
            }));
        } else {
            self.inner = Some(Box::new(CacheNode {
                data_path: data_path.to_path_buf(),
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
