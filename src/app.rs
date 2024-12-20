use once_cell::sync::OnceCell;
use std::{path::PathBuf, sync::RwLock};

const PATH: OnceCell<RwLock<PathBuf>> = OnceCell::new();

pub fn launch(path: &PathBuf) {
    init(path);
}

fn init(path: &PathBuf) {
    PATH.get_or_try_init(|| -> Result<RwLock<PathBuf>, ()> { Ok(RwLock::new(path.clone())) });
}
