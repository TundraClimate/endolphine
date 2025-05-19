use super::{Command, Component};

pub struct SideMenu {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    pub menu: std::sync::Arc<crate::menu::Menu>,
    pub is_opened: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

struct MenuToggle {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu: std::sync::Arc<crate::menu::Menu>,
    is_opened: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Command for MenuToggle {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        use std::sync::atomic::Ordering;

        if !self.is_opened.load(Ordering::Relaxed) || self.menu.is_enabled() {
            self.menu.toggle_enable();

            let mut app_state = self.app_state.write().unwrap();
            if matches!(app_state.mode, super::app::Mode::Menu) {
                app_state.mode = super::app::Mode::Normal;
            } else {
                app_state.mode = super::app::Mode::Menu;
            }
        }

        self.is_opened.fetch_not(Ordering::Relaxed);

        Ok(())
    }
}

struct MenuMove {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
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

        let mut app_state = self.app_state.write().unwrap();
        if matches!(app_state.mode, super::app::Mode::Menu) {
            app_state.mode = super::app::Mode::Normal;
        } else {
            app_state.mode = super::app::Mode::Menu;
        }

        Ok(())
    }
}

struct EnterFromMenu {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for EnterFromMenu {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        let cursor = &self.menu.cursor;
        if let Some(element) = self.menu.elements.get(cursor.current()) {
            let path = &element.path;

            if !path.is_dir() {
                crate::sys_log!("w", "Found the invalid Shortcut in MENU: {}", element.tag);
                crate::log!("\"{}\" is not Directory", element.tag);

                return Ok(());
            }

            let mut app_state = self.app_state.write().unwrap();

            app_state.path.swap(path)?;
            crate::sys_log!("i", "Change the open directory: {}", path.to_string_lossy());

            cursor.resize(crate::misc::child_files_len(path));
            cursor.reset();

            self.menu.toggle_enable();
            app_state.mode = super::app::Mode::Normal;
        }

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
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "M".parse()?,
                MenuToggle {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "M".parse()?,
                MenuToggle {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                    is_opened: self.is_opened.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "l".parse()?,
                EnterFromMenu {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                },
            )
        }

        Ok(())
    }
}
