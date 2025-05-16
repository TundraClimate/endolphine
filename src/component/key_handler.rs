use super::Component;
use super::app::AppState;
use super::root::RootState;

pub struct KeyHandler {
    pub root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Component for KeyHandler {
    fn on_tick(&self) -> Result<(), crate::Error> {
        let mut is_buffer_reset = true;

        {
            let root = self.root_state.read().unwrap();
            let current_mode = self.app_state.read().unwrap().mode;

            if root
                .mapping_registry
                .has_similar_map(root.key_buffer.get(), current_mode)
            {
                is_buffer_reset = false;

                if let Some(cmd) = root
                    .mapping_registry
                    .get(current_mode, root.key_buffer.get())
                {
                    is_buffer_reset = true;

                    let ctx = super::CommandContext {
                        prenum: root.key_buffer.prenum(),
                    };
                    cmd.run(ctx)?;
                }
            }
        }

        if is_buffer_reset {
            let mut root = self.root_state.write().unwrap();

            root.key_buffer.clear();
        }

        Ok(())
    }
}
