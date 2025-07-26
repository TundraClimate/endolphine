use crate::{
    canvas::Rect,
    component::{Cursor, CursorCache, Input, Selection},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        RwLock,
        atomic::{AtomicBool, AtomicU8, AtomicU16, AtomicUsize},
    },
};
use viks::Key;

pub struct State {
    pub work_dir: WorkingDir,
    pub mode: CurrentMode,
    pub key_buffer: KeyBuffer,
    pub term_size: TerminalRect,
    pub canvas_hashes: CanvasHashes,
    pub flag: FlagState,
    pub file_view: FileView,
    pub proc_counter: ProcessCounter,
    pub input: InputController,
    pub grep: Grep,
}

impl State {
    pub fn new(work_dir: PathBuf) -> State {
        Self {
            work_dir: WorkingDir::new(work_dir.clone()),
            mode: CurrentMode::new(),
            key_buffer: KeyBuffer::new(),
            term_size: TerminalRect::new(),
            canvas_hashes: CanvasHashes::new(),
            flag: FlagState::new(),
            file_view: FileView::new(work_dir.clone()),
            proc_counter: ProcessCounter::new(),
            input: InputController::new(),
            grep: Grep::new(),
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

    pub fn get(&self) -> PathBuf {
        self.wd.read().unwrap().clone()
    }

    pub fn store<P: AsRef<Path>>(&self, path: P) {
        let path = path.as_ref();

        *self.wd.write().unwrap() = path.to_path_buf();
    }
}

#[repr(u8)]
#[derive(PartialEq, Eq, Clone, Copy, Hash)]
pub enum Mode {
    Normal = 0,
    Visual = 1,
    Input = 2,
    Search = 3,
}

impl Mode {
    pub fn from_u8(i: u8) -> Option<Mode> {
        use std::mem;

        if (0..=3).contains(&i) {
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

    pub fn switch(&self, mode: Mode) {
        use std::sync::atomic::Ordering;

        self.now.store(mode as u8, Ordering::Relaxed);
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

    pub fn load(&self) -> Rect {
        use std::sync::atomic::Ordering;

        Rect {
            x: 0,
            y: 0,
            width: self.0.load(Ordering::Relaxed),
            height: self.1.load(Ordering::Relaxed),
        }
    }

    pub fn store(&self, cols: u16, rows: u16) {
        use std::sync::atomic::Ordering;

        self.0.store(cols, Ordering::Relaxed);
        self.1.store(rows, Ordering::Relaxed);
    }
}

pub struct CanvasHashes {
    hashes: RwLock<HashMap<u8, u64>>,
}

impl CanvasHashes {
    fn new() -> Self {
        Self {
            hashes: RwLock::new(HashMap::new()),
        }
    }

    pub fn update(&self, id: u8, hash: u64) -> Option<u64> {
        self.hashes.write().unwrap().insert(id, hash)
    }

    pub fn refresh(&self) {
        self.hashes.write().unwrap().clear();
    }
}

pub struct Flag {
    flag: AtomicBool,
}

impl Flag {
    fn new(default: bool) -> Self {
        Self {
            flag: AtomicBool::new(default),
        }
    }

    pub fn get(&self) -> bool {
        use std::sync::atomic::Ordering;

        self.flag.load(Ordering::Relaxed)
    }
}

pub struct FlagState {
    pub is_sidemenu_opened: Flag,
}

impl FlagState {
    fn new() -> Self {
        Self {
            is_sidemenu_opened: Flag::new(false),
        }
    }
}

pub struct FileView {
    pub cursor: Cursor,
    pub cursor_cache: CursorCache,
    pub selection: Selection,
}

impl FileView {
    fn new(wd: PathBuf) -> Self {
        use crate::misc;

        let s = Self {
            cursor: Cursor::default(),
            cursor_cache: CursorCache::new(),
            selection: Selection::new(),
        };

        s.cursor.resize(misc::child_files_len(&wd));

        s
    }
}

pub struct ProcessCounter {
    count: AtomicUsize,
}

impl ProcessCounter {
    fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }

    pub fn now(&self) -> usize {
        use std::sync::atomic::Ordering;

        self.count.load(Ordering::Relaxed)
    }

    pub fn increment(&self) {
        use std::sync::atomic::Ordering;

        self.count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement(&self) {
        use std::sync::atomic::Ordering;

        self.count.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct InputController {
    pub input: Input,
    tag: RwLock<Option<String>>,
}

impl InputController {
    fn new() -> Self {
        Self {
            input: Input::new(),
            tag: RwLock::new(None),
        }
    }

    pub fn is_enable(&self) -> bool {
        self.tag.read().unwrap().is_some()
    }

    pub fn enable(&self, tag: &str) {
        *self.tag.write().unwrap() = Some(tag.to_string());
    }

    pub fn disable(&self) {
        *self.tag.write().unwrap() = None;
    }

    pub fn tag(&self) -> Option<String> {
        self.tag.read().unwrap().clone()
    }
}

pub struct Grep {
    buf: RwLock<String>,
}

impl Grep {
    fn new() -> Self {
        Self {
            buf: RwLock::new(String::new()),
        }
    }

    pub fn load(&self) -> String {
        self.buf.read().unwrap().clone()
    }

    pub fn update(&self, new: String) {
        *self.buf.write().unwrap() = new;
    }

    pub fn clear(&self) {
        self.buf.write().unwrap().clear()
    }
}
