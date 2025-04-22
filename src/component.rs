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

struct App(Vec<Box<dyn Component>>);

impl Component for App {}

pub fn components() -> Box<dyn Component> {
    Box::new(Root::with_state(|root_state| {
        vec![
            Box::new(KeyReader {
                root_state: root_state.clone(),
            }),
            Box::new(App(vec![])),
        ]
    }))
}
