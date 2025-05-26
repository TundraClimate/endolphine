use super::{Command, Component};

pub struct SideMenu {
    root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu_rect: std::sync::Arc<std::sync::RwLock<crate::canvas_impl::Rect>>,
    menu_canvas: std::sync::Arc<std::sync::RwLock<MenuCanvas>>,
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl SideMenu {
    pub fn new(
        root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
        app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
        menu_rect: std::sync::Arc<std::sync::RwLock<crate::canvas_impl::Rect>>,
        menu: std::sync::Arc<crate::menu::Menu>,
    ) -> Self {
        Self {
            root_state,
            app_state,
            menu_canvas: std::sync::Arc::new(std::sync::RwLock::new(MenuCanvas::new_with_init(
                *menu_rect.clone().read().unwrap(),
            ))),
            menu_rect,
            menu,
        }
    }
}

struct MenuToggle {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for MenuToggle {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        let mut app_state = self.app_state.write().unwrap();

        if !app_state.is_menu_opened.load() || self.menu.is_enabled() {
            self.menu.toggle_enable();

            if matches!(app_state.mode, super::app::Mode::Menu) {
                app_state.mode = super::app::Mode::Normal;
            } else {
                app_state.mode = super::app::Mode::Menu;
            }
        }

        app_state.is_menu_opened.update_not();

        Ok(())
    }
}

struct MenuMove {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for MenuMove {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        let mut app_state = self.app_state.write().unwrap();

        if !app_state.is_menu_opened.load() {
            app_state.is_menu_opened.update_not();
        }

        self.menu.toggle_enable();

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

struct MenuCanvas {
    canvas: crate::canvas_impl::Canvas,
    prev_rect: crate::canvas_impl::Rect,
}

impl MenuCanvas {
    fn init(&mut self) {
        self.canvas.set_bg(crossterm::style::Color::Blue);
        self.canvas.set_fg(crossterm::style::Color::White);
        self.canvas.fill();
    }

    fn new_with_init(rect: crate::canvas_impl::Rect) -> Self {
        let mut c = Self {
            canvas: crate::canvas_impl::Canvas::from(rect),
            prev_rect: rect,
        };

        c.init();

        c
    }

    fn has_rect_update(&self, rect: crate::canvas_impl::Rect) -> bool {
        self.prev_rect != rect
    }

    fn draw(&self) {
        self.canvas.print(1, 0, "Hi Menu");
    }

    fn reset_size_with_init(&mut self, rect: crate::canvas_impl::Rect) {
        self.canvas = crate::canvas_impl::Canvas::from(rect);
        self.prev_rect = rect;
        self.init();
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
                },
            );
            registry.register_key(
                Mode::Visual,
                "M".parse()?,
                MenuToggle {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "M".parse()?,
                MenuToggle {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "m".parse()?,
                MenuMove {
                    app_state: self.app_state.clone(),
                    menu: self.menu.clone(),
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

    fn on_tick(&self) -> Result<(), crate::Error> {
        let rect = *self.menu_rect.read().unwrap();
        if self.menu_canvas.read().unwrap().has_rect_update(rect) {
            self.menu_canvas.write().unwrap().reset_size_with_init(rect);
        }

        self.menu_canvas.read().unwrap().draw();

        Ok(())
    }

    fn on_resize(&self, _size: (u16, u16)) -> Result<(), crate::Error> {
        self.menu_canvas
            .write()
            .unwrap()
            .reset_size_with_init(*self.menu_rect.read().unwrap());

        Ok(())
    }
}
