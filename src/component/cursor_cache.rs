use std::{
    path::{Path, PathBuf},
    sync::RwLock,
};

#[derive(Clone, Debug)]
struct CacheNode {
    data_path: PathBuf,
    inner: Option<Box<CacheNode>>,
}

#[derive(Debug)]
pub struct CursorCache {
    inner: RwLock<Option<Box<CacheNode>>>,
}

impl CursorCache {
    pub fn new() -> Self {
        CursorCache {
            inner: RwLock::new(None),
        }
    }

    pub fn wrap_node(&self, data_path: &Path) {
        let mut lock = self.inner.write().unwrap();

        if let Some(inner) = &*lock {
            *lock = Some(Box::new(CacheNode {
                data_path: data_path.to_path_buf(),
                inner: Some(Box::new(*inner.clone())),
            }));
        } else {
            *lock = Some(Box::new(CacheNode {
                data_path: data_path.to_path_buf(),
                inner: None,
            }))
        }
    }

    pub fn unwrap_surface(&self) {
        let mut lock = self.inner.write().unwrap();

        if let Some(inner) = &*lock {
            *lock = inner.inner.clone();
        }
    }

    pub fn reset(&self) {
        *self.inner.write().unwrap() = None;
    }

    pub fn inner_equal(&self, data_path: &PathBuf) -> bool {
        matches!(&*self.inner.read().unwrap(), Some(inner) if &inner.data_path == data_path)
    }
}
