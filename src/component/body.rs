use super::Command;
use super::Component;
use super::app::AppState;
use super::root::RootState;

#[derive(Default)]
struct Selection {
    inner: Option<(usize, usize)>,
}

impl Selection {
    fn is_active(&self) -> bool {
        self.inner.is_some()
    }

    fn select_area(&mut self, other: usize) {
        if let Some((base, _)) = self.inner {
            self.inner = Some((base, other));
        }
    }
}

#[derive(Default)]
pub struct BodyState {
    cursor: crate::cursor::Cursor,
    selection: Selection,
}

pub struct Body {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    inner: Vec<Box<dyn Component>>,
}

impl Body {
    pub fn with_state<
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

struct MoveDown {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for MoveDown {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();

        state.cursor.shift_p(self.prenum);

        if state.selection.is_active() {
            let cursor_pos = state.cursor.current();
            state.selection.select_area(cursor_pos);
        }

        Ok(())
    }
}

struct MoveUp {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for MoveUp {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();

        state.cursor.shift_n(self.prenum);

        if state.selection.is_active() {
            let cursor_pos = state.cursor.current();
            state.selection.select_area(cursor_pos);
        }

        Ok(())
    }
}

impl Component for Body {
    fn on_init(&self) -> Result<(), crate::Error> {
        use super::app::Mode;

        {
            let mut lock = self.root_state.write().unwrap();
            let prenum = lock.key_buffer.prenum();
            let registry = &mut lock.mapping_registry;

            registry.register_key(
                Mode::Normal,
                "j".parse()?,
                MoveDown {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Normal,
                "k".parse()?,
                MoveUp {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
        }

        Ok(())
    }
}
