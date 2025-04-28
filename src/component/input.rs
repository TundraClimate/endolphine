use super::{Command, Component};

pub struct Input {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

struct CompleteInput {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CompleteInput {
    fn run(&self) -> Result<(), crate::Error> {
        let (action, content) = {
            let mut lock = self.app_state.write().unwrap();
            let input = &mut lock.input;

            input.complete_input();

            (input.drain_action(), input.drain_storage())
        };

        let rc = self.app_state.clone();

        tokio::task::spawn_blocking(move || {
            let Some(content) = content else { return };

            if let Some(action) = action {
                let mut lock = rc.write().unwrap();
                let proc_counter = &mut lock.process_counter;

                proc_counter.up();
                // HANDLER HERE
                proc_counter.down();
            }
        });

        Ok(())
    }
}

struct CancelInput {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CancelInput {
    fn run(&self) -> Result<(), crate::Error> {
        self.app_state.write().unwrap().input.disable();

        Ok(())
    }
}

struct InputCursorNext {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputCursorNext {
    fn run(&self) -> Result<(), crate::Error> {
        self.app_state.write().unwrap().input.cursor_right();

        Ok(())
    }
}

struct InputCursorPrev {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputCursorPrev {
    fn run(&self) -> Result<(), crate::Error> {
        self.app_state.write().unwrap().input.cursor_left();

        Ok(())
    }
}

impl Component for Input {
    fn on_init(&self) -> Result<(), crate::Error> {
        use super::app::Mode;

        {
            let mut lock = self.root_state.write().unwrap();
            let registry = &mut lock.mapping_registry;

            registry.register_key(
                Mode::Input,
                "<CR>".parse()?,
                CompleteInput {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<ESC>".parse()?,
                CancelInput {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-l>".parse()?,
                InputCursorNext {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-h>".parse()?,
                InputCursorPrev {
                    app_state: self.app_state.clone(),
                },
            );
        }

        Ok(())
    }
}
