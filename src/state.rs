use crate::proc::Runnable;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{RwLock, atomic::AtomicU8},
};
use viks::Keymap;

pub struct State {
    work_dir: WorkingDir,
    mode: CurrentMode,
    mapping: KeymapRegistry,
}

impl State {
    pub fn new(work_dir: PathBuf) -> State {
        Self {
            work_dir: WorkingDir::new(work_dir),
            mode: CurrentMode::new(),
            mapping: KeymapRegistry::new(),
        }
    }
}

struct WorkingDir {
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
pub enum Mode {
    Normal = 0,
    Visual = 1,
}

struct CurrentMode {
    now: AtomicU8,
}

impl CurrentMode {
    fn new() -> Self {
        Self {
            now: AtomicU8::new(0),
        }
    }
}

pub struct KeymapRegistry {
    user_defined: RwLock<HashMap<(u8, String), Box<dyn Runnable>>>,
}

impl KeymapRegistry {
    fn new() -> Self {
        Self {
            user_defined: RwLock::new(HashMap::new()),
        }
    }

    fn register<R: Runnable + 'static>(
        register: &mut HashMap<(u8, String), Box<dyn Runnable>>,
        mode: Mode,
        map: viks::Result<Keymap>,
        exec: R,
    ) {
        register.insert(
            (
                mode as u8,
                map.expect("invalid mapping found: KeymapRegistry")
                    .to_string(),
            ),
            Box::new(exec),
        );
    }

    pub fn constants() -> &'static HashMap<(u8, String), Box<dyn Runnable>> {
        use crate::{proc::Command, tui};
        use std::sync::LazyLock;

        static BASE_MAPPING: LazyLock<HashMap<(u8, String), Box<dyn Runnable>>> =
            LazyLock::new(|| {
                let mut register = HashMap::new();

                KeymapRegistry::register(
                    &mut register,
                    Mode::Normal,
                    Keymap::new("ZZ"),
                    Command(|_| tui::close()),
                );

                register
            });

        &BASE_MAPPING
    }
}
