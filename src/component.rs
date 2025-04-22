pub trait Component: Send + Sync {
    fn on_init(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        Ok(())
    }
}

#[derive(Default)]
struct KeyBuffer {
    inner: Vec<crate::key::Key>,
}

impl KeyBuffer {
    fn push(&mut self, key: crate::key::Key) {
        self.inner.push(key);
    }
}

#[derive(Default)]
struct RootState {
    key_buffer: KeyBuffer,
}

struct Root(Vec<Box<dyn Component>>);

impl Root {
    fn with_state<
        F: FnOnce(std::sync::Arc<std::sync::RwLock<RootState>>) -> Vec<Box<dyn Component>>,
    >(
        f: F,
    ) -> Self {
        use std::sync::{Arc, RwLock};

        let root_state = Arc::new(RwLock::new(RootState::default()));

        Self(f(root_state))
    }
}

impl Component for Root {
    fn on_init(&self) -> Result<(), crate::Error> {
        self.0.iter().try_for_each(|inner| inner.on_init())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        self.0.iter().try_for_each(|inner| inner.on_tick())
    }
}

struct KeyReader {
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
}

impl Component for KeyReader {
    fn on_init(&self) -> Result<(), crate::Error> {
        let state = self.root_state.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                    let key = crate::key::Key::from_keyevent(&key_event);

                    state.write().unwrap().key_buffer.push(key);
                }
            }
        });

        Ok(())
    }
}

struct CurrentPath {
    inner: std::path::PathBuf,
}

impl CurrentPath {
    fn swap(&mut self, path: &std::path::Path) -> Result<(), crate::Error> {
        if !path.is_dir() {
            return Err(crate::Error::InvalidArgument(
                path.to_string_lossy().to_string(),
            ));
        }

        self.inner = path.to_path_buf();

        Ok(())
    }

    fn get(&self) -> &std::path::PathBuf {
        &self.inner
    }
}

impl Default for CurrentPath {
    fn default() -> Self {
        use clap::Parser;

        Self {
            inner: crate::Args::parse().path,
        }
    }
}

enum Mode {
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
struct ProcessCounter(usize);

impl ProcessCounter {
    fn up(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    fn down(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }

    fn now(&self) -> usize {
        self.0
    }
}

#[derive(Default)]
struct AppState {
    path: CurrentPath,
    pub is_render: bool,
    pub mode: Mode,
    process_counter: ProcessCounter,
}

struct App {
    state: std::sync::Arc<std::sync::RwLock<AppState>>,
    inner: Vec<Box<dyn Component>>,
}

impl App {
    fn with_state<
        F: FnOnce(std::sync::Arc<std::sync::RwLock<AppState>>) -> Vec<Box<dyn Component>>,
    >(
        f: F,
    ) -> Self {
        use std::sync::{Arc, RwLock};

        let app_state = Arc::new(RwLock::new(AppState::default()));

        Self {
            state: app_state.clone(),
            inner: f(app_state.clone()),
        }
    }
}

impl Component for App {
    fn on_init(&self) -> Result<(), crate::Error> {
        self.inner.iter().try_for_each(|inner| inner.on_init())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        self.inner.iter().try_for_each(|inner| inner.on_tick())
    }
}

pub fn components() -> Box<dyn Component> {
    Box::new(Root::with_state(|root_state| {
        vec![
            Box::new(KeyReader {
                root_state: root_state.clone(),
            }),
            Box::new(App::with_state(|_app_state| vec![])),
        ]
    }))
}
