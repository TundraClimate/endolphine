use super::Command;
use super::Component;
use super::root::RootState;

pub struct CurrentPath {
    inner: std::path::PathBuf,
}

impl CurrentPath {
    pub fn swap(&mut self, path: &std::path::Path) -> Result<(), crate::Error> {
        if !path.is_dir() {
            return Err(crate::Error::InvalidArgument(
                path.to_string_lossy().to_string(),
            ));
        }

        self.inner = path.to_path_buf();

        Ok(())
    }

    pub fn get(&self) -> &std::path::PathBuf {
        &self.inner
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Mode {
    Normal,
    Visual,
    Input,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Default)]
pub struct ProcessCounter(usize);

impl ProcessCounter {
    pub fn up(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    pub fn down(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    fn now(&self) -> usize {
        self.0
    }
}

pub struct Config {
    inner: crate::config::Config,
}

impl Config {
    fn new_with_init() -> Self {
        Self {
            inner: crate::config::file_path()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|c| toml::from_str(&c).ok())
                .unwrap_or_else(|| {
                    crate::sys_log!("w", "load config.toml failed, use the default config");
                    crate::config::Config::default()
                }),
        }
    }

    pub fn get(&self) -> &crate::config::Config {
        &self.inner
    }
}

pub struct AppState {
    pub path: CurrentPath,
    pub config: Config,
    pub is_render: bool,
    pub mode: Mode,
    pub process_counter: ProcessCounter,
}

pub struct App {
    state: std::sync::Arc<std::sync::RwLock<AppState>>,
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    inner: Vec<Box<dyn Component>>,
}

impl App {
    pub fn with_state<
        F: FnOnce(std::sync::Arc<std::sync::RwLock<AppState>>) -> Vec<Box<dyn Component>>,
    >(
        root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
        f: F,
    ) -> Self {
        use clap::Parser;
        use std::sync::{Arc, RwLock};

        let path = crate::Args::parse().path;

        let path = match path.canonicalize().map_err(|_| {
            crate::Error::InvalidArgument(format!(
                "{} is cannot canonicalize",
                path.to_string_lossy()
            ))
        }) {
            Ok(path) => path,
            Err(e) => {
                e.handle();
                unreachable!()
            }
        };

        let app_state = Arc::new(RwLock::new(AppState {
            path: CurrentPath { inner: path },
            config: Config::new_with_init(),
            is_render: true,
            mode: Mode::default(),
            process_counter: ProcessCounter::default(),
        }));

        Self {
            state: app_state.clone(),
            root_state,
            inner: f(app_state.clone()),
        }
    }
}

struct ExitApp;

impl Command for ExitApp {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        crate::app::disable_tui()?;

        crate::sys_log!("i", "Endolphine close successfully");

        std::process::exit(0)
    }
}

impl Component for App {
    fn on_init(&self) -> Result<(), crate::Error> {
        {
            let mut lock = self.root_state.write().unwrap();
            let registry = &mut lock.mapping_registry;

            registry.register_key(Mode::Normal, "ZZ".parse()?, ExitApp);
            registry.register_key(Mode::Visual, "ZZ".parse()?, ExitApp);
        }

        self.inner.iter().try_for_each(|inner| inner.on_init())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        self.inner.iter().try_for_each(|inner| inner.on_tick())
    }
}
