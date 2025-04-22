use super::Component;
use super::app::AppState;
use super::root::RootState;

#[derive(Default)]
pub struct BodyState {
    cursor: crate::cursor::Cursor,
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

impl Component for Body {}
