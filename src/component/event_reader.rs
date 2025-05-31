use super::Component;
use super::root::RootState;

pub struct EventReader {
    pub root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
}

impl Component for EventReader {
    fn on_init(&self) -> Result<(), crate::Error> {
        let state = self.root_state.clone();

        tokio::task::spawn_blocking(move || {
            loop {
                match crossterm::event::read() {
                    Ok(crossterm::event::Event::Key(key_event)) => {
                        let key = crate::key::Key::from_keyevent(&key_event);

                        state.write().unwrap().key_buffer.push(key);
                    }
                    Ok(crossterm::event::Event::Resize(col, row)) => {
                        state.write().unwrap().size_status.update(col, row);
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}
