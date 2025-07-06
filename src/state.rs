use std::{
    path::PathBuf,
    sync::{
        RwLock,
        atomic::{AtomicU8, AtomicU16},
    },
};
use viks::Key;

pub struct State {
    pub work_dir: WorkingDir,
    pub mode: CurrentMode,
    pub key_buffer: KeyBuffer,
    pub term_size: TerminalRect,
}

impl State {
    pub fn new(work_dir: PathBuf) -> State {
        Self {
            work_dir: WorkingDir::new(work_dir),
            mode: CurrentMode::new(),
            key_buffer: KeyBuffer::new(),
            term_size: TerminalRect::new(),
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

pub struct TerminalRect(AtomicU16, AtomicU16);

impl TerminalRect {
    fn new() -> Self {
        use crossterm::terminal;

        let Ok((cols, rows)) = terminal::size() else {
            panic!("Couldn't get a tty size");
        };

        Self(AtomicU16::new(cols), AtomicU16::new(rows))
    }

    pub fn store(&self, cols: u16, rows: u16) {
        use std::sync::atomic::Ordering;

        self.0.store(cols, Ordering::Relaxed);
        self.1.store(rows, Ordering::Relaxed);
    }
}
