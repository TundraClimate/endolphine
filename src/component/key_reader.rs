use super::Component;
use super::root::RootState;

pub struct KeyReader {
    pub root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
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
