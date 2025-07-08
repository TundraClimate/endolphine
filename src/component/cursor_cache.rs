use std::path::{Path, PathBuf};

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
    pub fn new() -> Self {
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
