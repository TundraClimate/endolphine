use std::{path::PathBuf, sync::RwLock};

pub struct State {
    work_dir: WorkingDir,
}

impl State {
    pub fn new(work_dir: PathBuf) -> State {
        Self {
            work_dir: WorkingDir::new(work_dir),
        }
    }
}

struct WorkingDir {
    wd: RwLock<PathBuf>,
}

impl WorkingDir {
    fn new(path: PathBuf) -> Self {
        WorkingDir {
            wd: RwLock::new(path),
        }
    }
}
