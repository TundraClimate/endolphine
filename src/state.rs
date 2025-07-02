use std::{
    path::PathBuf,
    sync::{RwLock, atomic::AtomicU8},
};
use viks::Key;

pub struct State {
    pub work_dir: WorkingDir,
    pub mode: CurrentMode,
    pub key_buffer: KeyBuffer,
}

impl State {
    pub fn new(work_dir: PathBuf) -> State {
        Self {
            work_dir: WorkingDir::new(work_dir),
            mode: CurrentMode::new(),
            key_buffer: KeyBuffer::new(),
        }
    }
}

pub struct WorkingDir {
    wd: RwLock<PathBuf>,
}

impl WorkingDir {
    fn new(path: PathBuf) -> Self {
        Self {
            wd: RwLock::new(path),
        }
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum Mode {
    Normal = 0,
    Visual = 1,
}

impl Mode {
    pub fn from_u8(i: u8) -> Option<Mode> {
        use std::mem;

        if i == 0 || i == 1 {
            Some(unsafe { mem::transmute::<u8, Mode>(i) })
        } else {
            None
        }
    }
}

pub struct CurrentMode {
    now: AtomicU8,
}

impl CurrentMode {
    fn new() -> Self {
        Self {
            now: AtomicU8::new(0),
        }
    }

    pub fn get(&self) -> Mode {
        use std::sync::atomic::Ordering;

        Mode::from_u8(self.now.load(Ordering::Relaxed)).expect("Invalid mode detected")
    }
}

pub struct KeyBuffer {
    buffer: RwLock<Vec<Key>>,
}

impl KeyBuffer {
    fn new() -> Self {
        KeyBuffer {
            buffer: RwLock::new(vec![]),
        }
    }

    pub fn push(&self, key: Key) {
        self.buffer.write().unwrap().push(key);
    }

    pub fn append(&self, keys: &mut Vec<Key>) {
        self.buffer.write().unwrap().append(keys);
    }

    pub fn drain(&self) -> Vec<Key> {
        self.buffer.write().unwrap().drain(..).collect()
    }
}
