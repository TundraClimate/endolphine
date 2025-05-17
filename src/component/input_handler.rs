use super::Component;

pub struct InputHandler {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Component for InputHandler {
    fn on_tick(&self) -> Result<(), crate::Error> {
        if matches!(self.app_state.read().unwrap().mode, super::app::Mode::Input) {
            return Ok(());
        }

        let (action, content) = {
            let mut lock = self.body_state.write().unwrap();
            let input = &mut lock.input;

            (input.drain_action(), input.drain_storage())
        };

        let Some(content) = content else {
            return Ok(());
        };

        let app_state = self.app_state.clone();
        let body_state = self.body_state.clone();
        let prenum = self.root_state.read().unwrap().key_buffer.prenum();
        let ctx = super::CommandContext { prenum };

        tokio::task::spawn_blocking(move || {
            if let Some(action) = action {
                {
                    let mut lock = app_state.write().unwrap();
                    let proc_counter = &mut lock.process_counter;

                    proc_counter.up();
                }

                {
                    if let Err(e) =
                        super::body::run_input_task(&action, &content, &body_state, &app_state, ctx)
                    {
                        e.handle();
                    };
                }

                {
                    let mut lock = app_state.write().unwrap();
                    let proc_counter = &mut lock.process_counter;

                    proc_counter.down();
                }
            }
        });

        Ok(())
    }
}
