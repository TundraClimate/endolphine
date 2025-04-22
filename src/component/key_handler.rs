use super::Component;
use super::app::AppState;
use super::root::RootState;

pub struct KeyHandler {
    pub root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
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
