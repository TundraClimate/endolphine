use super::{Command, Component};

pub struct SideMenu {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub menu: std::sync::Arc<crate::menu::Menu>,
    pub is_opened: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

struct MenuToggle {
    menu: std::sync::Arc<crate::menu::Menu>,
    is_opened: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Command for MenuToggle {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        use std::sync::atomic::Ordering;

        if !self.is_opened.load(Ordering::Relaxed) || self.menu.is_enabled() {
            self.menu.toggle_enable();
        }

        self.is_opened.fetch_not(Ordering::Relaxed);

        Ok(())
    }
}

struct MenuMove {
    menu: std::sync::Arc<crate::menu::Menu>,
    is_opened: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Command for MenuMove {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        use std::sync::atomic::Ordering;

        if !self.is_opened.load(Ordering::Relaxed) {
            self.is_opened.fetch_not(Ordering::Relaxed);
        }

        self.menu.toggle_enable();

        Ok(())
    }
}

impl Component for SideMenu {
    fn on_init(&self) -> Result<(), crate::Error> {
        use super::app::Mode;

        {
            let mut root = self.root_state.write().unwrap();
            let registry = &mut root.mapping_registry;

            registry.register_key(
                Mode::Normal,
                "M".parse()?,
                MenuToggle {
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "M".parse()?,
                MenuToggle {
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "m".parse()?,
                MenuMove {
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "m".parse()?,
                MenuMove {
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
        }

        Ok(())
    }
}
