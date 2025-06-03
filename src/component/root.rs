use super::Command;
use super::Component;
use super::app::Mode;

#[derive(Default)]
pub struct KeyBuffer {
    pub inner: Vec<crate::key::Key>,
}

impl KeyBuffer {
    pub fn get(&self) -> &Vec<crate::key::Key> {
        &self.inner
    }

    pub fn push(&mut self, key: crate::key::Key) {
        self.inner.push(key);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn prenum(&self) -> Option<usize> {
        let prenum = self
            .inner
            .clone()
            .into_iter()
            .take_while(crate::key::Key::is_digit)
            .map(|k| k.as_num())
            .collect::<Vec<_>>();
        let mut sum = 0usize;

        for (i, k) in prenum.into_iter().rev().enumerate() {
            sum += (k - 48) as usize * (10usize.pow(i as u32));
        }

        if sum == 0 { None } else { Some(sum) }
    }
}

#[derive(Default)]
pub struct MappingRegistry {
    inner: std::collections::HashMap<(u8, String), Box<dyn Command>>,
}

impl MappingRegistry {
    pub fn register_key<C: Command + 'static>(
        &mut self,
        mode: Mode,
        keymap: crate::key::Keymap,
        cmd: C,
    ) {
        self.inner
            .insert((mode as u8, keymap.to_string()), Box::new(cmd));
    }

    pub fn has_map(&self, buf: &[crate::key::Key], mode: super::app::Mode) -> bool {
        if buf.is_empty() || buf.iter().all(crate::key::Key::is_digit) {
            return false;
        }

        let buf = buf.iter().skip_while(|k| k.is_digit()).collect::<Vec<_>>();

        let mode = mode as u8;

        self.inner.keys().any(|(rmode, keymap)| {
            buf.len() == keymap.len()
                && mode == *rmode
                && buf.iter().enumerate().all(|(i, k)| {
                    keymap
                        .as_str()
                        .parse::<crate::key::Keymap>()
                        .is_ok_and(|key| key.as_vec().get(i) == Some(k))
                })
        })
    }

    pub fn has_similar_map(&self, buf: &[crate::key::Key], mode: Mode) -> bool {
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

    pub fn get(&self, mode: Mode, keymap: &[crate::key::Key]) -> Option<&dyn Command> {
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
            .map(|cmd| &**cmd)
    }

    pub fn get_pure(&self, mode: Mode, keymap: &[crate::key::Key]) -> Option<&dyn Command> {
        self.inner
            .get(&(mode as u8, crate::key::Keymap::new(keymap).to_string()))
            .map(|cmd| &**cmd)
    }
}

#[derive(Default)]
pub struct SizeStatus {
    columns: u16,
    rows: u16,
    hook: crate::hook::Hook,
}

impl SizeStatus {
    pub fn update(&mut self, cols: u16, rows: u16) {
        self.columns = cols;
        self.rows = rows;
        self.hook.pull();
    }
}

#[derive(Default)]
pub struct RootState {
    pub key_buffer: KeyBuffer,
    pub mapping_registry: MappingRegistry,
    pub size_status: SizeStatus,
}

pub struct Root {
    state: std::sync::Arc<std::sync::RwLock<RootState>>,
    base_rect: std::sync::Arc<std::sync::RwLock<crate::canvas_impl::Rect>>,
    inner: Vec<Box<dyn Component>>,
}

impl Root {
    fn split_rect(
        rect: crate::canvas_impl::Rect,
    ) -> (crate::canvas_impl::Rect, crate::canvas_impl::Rect) {
        use crate::canvas_impl::LayoutSpec;

        let split = rect.split_horizontal(vec![LayoutSpec::Fill, LayoutSpec::Min(1)]);

        (split[0], split[1])
    }

    pub fn with_state<
        F: FnOnce(
            std::sync::Arc<std::sync::RwLock<RootState>>,
            std::sync::Arc<std::sync::RwLock<crate::canvas_impl::Rect>>,
        ) -> Vec<Box<dyn Component>>,
    >(
        f: F,
    ) -> Self {
        use std::sync::{Arc, RwLock};

        let root_state = Arc::new(RwLock::new(RootState::default()));

        let size = crossterm::terminal::size().unwrap_or((0, 0));
        let rect = crate::canvas_impl::Rect::new(0, 0, size.0, size.1);
        let split = Root::split_rect(rect);
        let base_rect = Arc::new(RwLock::new(split.0));

        Self {
            state: root_state.clone(),
            inner: f(root_state, base_rect.clone()),
            base_rect,
        }
    }
}

impl Component for Root {
    fn on_init(&self) -> Result<(), crate::Error> {
        self.inner.iter().try_for_each(|inner| inner.on_init())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        {
            let state = self.state.read().unwrap();

            if let Some(res) = state
                .size_status
                .hook
                .effect(|| self.on_resize((state.size_status.columns, state.size_status.rows)))
            {
                res?;
            }
        }

        self.inner.iter().try_for_each(|inner| inner.on_tick())
    }

    fn on_resize(&self, size: (u16, u16)) -> Result<(), crate::Error> {
        {
            let size = crossterm::terminal::size().unwrap_or((0, 0));
            let rect = crate::canvas_impl::Rect::new(0, 0, size.0, size.1);

            *self.base_rect.write().unwrap() = Root::split_rect(rect).0;
        }

        self.inner
            .iter()
            .try_for_each(|inner| inner.on_resize(size))
    }
}
