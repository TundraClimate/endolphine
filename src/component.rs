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

    fn clear(&mut self) {
        self.inner.clear();
    }
}

#[derive(Default)]
struct MappingRegistry {
    inner: std::collections::HashMap<(u8, String), Box<dyn crate::command::Command>>,
}

impl MappingRegistry {
    fn register_key<C: crate::command::Command + 'static>(
        &mut self,
        mode: Mode,
        keymap: crate::key::Keymap,
        cmd: C,
    ) {
        self.inner
            .insert((mode as u8, keymap.to_string()), Box::new(cmd));
    }

    fn has_similar_map(&self, buf: &[crate::key::Key], mode: Mode) -> bool {
        if buf.is_empty() {
            return false;
        }

        if buf.iter().all(crate::key::Key::is_digit) {
            return true;
        }

        let buf = buf.iter().skip_while(|k| k.is_digit()).collect::<Vec<_>>();

        let mode = mode as u8;

        self.inner.keys().any(|(rmode, keymap)| {
            buf.len() <= keymap.len()
                && mode == *rmode
                && buf.iter().enumerate().all(|(i, k)| {
                    keymap
                        .as_str()
                        .parse::<crate::key::Keymap>()
                        .is_ok_and(|key| key.as_vec().get(i) == Some(k))
                })
        })
    }

    fn eval_keymap(
        &self,
        mode: Mode,
        keymap: &[crate::key::Key],
    ) -> Option<Result<(), crate::Error>> {
        let keymap = keymap
            .iter()
            .skip_while(|k| k.is_digit())
            .cloned()
            .collect::<Vec<crate::key::Key>>();

        self.inner
            .get(&(
                mode as u8,
                crate::key::Keymap::new(keymap.as_slice()).to_string(),
            ))
            .map(|cmd| cmd.run())
    }
}

#[derive(Default)]
struct RootState {
    key_buffer: KeyBuffer,
    mapping_registry: MappingRegistry,
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

#[repr(u8)]
#[derive(Clone, Copy)]
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

struct KeyHandler {
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Component for KeyHandler {
    fn on_tick(&self) -> Result<(), crate::Error> {
        let mut root = self.root_state.write().unwrap();
        let app = self.app_state.read().unwrap();

        if !root
            .mapping_registry
            .has_similar_map(&root.key_buffer.inner, app.mode)
        {
            root.key_buffer.clear();

            return Ok(());
        }

        if let Some(cmd_res) = root
            .mapping_registry
            .eval_keymap(app.mode, &root.key_buffer.inner)
        {
            root.key_buffer.clear();
            cmd_res?;
        }

        Ok(())
    }
}

#[derive(Default)]
struct BodyState {
    cursor: crate::cursor::Cursor,
}

struct Body {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    inner: Vec<Box<dyn Component>>,
}

impl Body {
    fn with_state<
        F: FnOnce(std::sync::Arc<std::sync::RwLock<BodyState>>) -> Vec<Box<dyn Component>>,
    >(
        app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
        root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
        f: F,
    ) -> Self {
        let body_state = std::sync::Arc::new(std::sync::RwLock::new(BodyState::default()));

        Self {
            state: body_state.clone(),
            app_state,
            root_state,
            inner: f(body_state.clone()),
        }
    }
}

impl Component for Body {
    fn on_init(&self) -> Result<(), crate::Error> {
        {
            let mut lock = self.root_state.write().unwrap();
            lock.mapping_registry.register_key(
                Mode::Normal,
                "ZZ".parse()?,
                crate::command::ExitApp,
            );
        }

        Ok(())
    }
}

pub fn components() -> Box<dyn Component> {
    Box::new(Root::with_state(|root_state| {
        vec![
            Box::new(KeyReader {
                root_state: root_state.clone(),
            }),
            Box::new(App::with_state(|app_state| {
                vec![
                    Box::new(KeyHandler {
                        root_state: root_state.clone(),
                        app_state: app_state.clone(),
                    }),
                    Box::new(Body::with_state(
                        app_state.clone(),
                        root_state.clone(),
                        |_body_state| vec![],
                    )),
                ]
            })),
        ]
    }))
}
