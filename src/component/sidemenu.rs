use super::{Command, Component};

pub struct SideMenu {
    root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    menu_rect: std::sync::Arc<std::sync::RwLock<crate::canvas::Rect>>,
    menu_canvas: std::sync::Arc<std::sync::RwLock<MenuCanvas>>,
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl SideMenu {
    pub fn new(
        root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
        app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
        menu_rect: std::sync::Arc<std::sync::RwLock<crate::canvas::Rect>>,
        menu: std::sync::Arc<crate::menu::Menu>,
    ) -> Self {
        Self {
            root_state,
            menu_canvas: std::sync::Arc::new(std::sync::RwLock::new(MenuCanvas::new_with_init(
                app_state.clone(),
                *menu_rect.clone().read().unwrap(),
            ))),
            app_state,
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

struct CursorUp {
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for CursorUp {
    fn run(&self, ctx: super::CommandContext) -> Result<(), crate::Error> {
        let cursor = &self.menu.cursor;

        cursor.shift_n(ctx.prenum.unwrap_or(1));

        Ok(())
    }
}

struct CursorDown {
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for CursorDown {
    fn run(&self, ctx: super::CommandContext) -> Result<(), crate::Error> {
        let cursor = &self.menu.cursor;

        cursor.shift_p(ctx.prenum.unwrap_or(1));

        Ok(())
    }
}

struct CursorToTop {
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for CursorToTop {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        let cursor = &self.menu.cursor;

        cursor.reset();

        Ok(())
    }
}

struct CursorToBottom {
    menu: std::sync::Arc<crate::menu::Menu>,
}

impl Command for CursorToBottom {
    fn run(&self, _ctx: super::CommandContext) -> Result<(), crate::Error> {
        let cursor = &self.menu.cursor;

        cursor.shift_p(cursor.len());

        Ok(())
    }
}

struct MenuCanvas {
    canvas: crate::canvas::Canvas,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    key: String,
    prev_rect: crate::canvas::Rect,
}

impl MenuCanvas {
    fn init(&mut self) {
        let config = self.app_state.read().unwrap().config.get().scheme();

        self.canvas.set_bg(config.bg_focused);
        self.canvas.set_fg(config.fg_focused);
        self.canvas.fill();
    }

    fn new_with_init(
        app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
        rect: crate::canvas::Rect,
    ) -> Self {
        let mut c = Self {
            canvas: crate::canvas::Canvas::from(rect),
            app_state,
            key: String::new(),
            prev_rect: rect,
        };

        c.init();

        c
    }

    fn has_rect_update(&self, rect: crate::canvas::Rect) -> bool {
        self.prev_rect != rect
    }

    fn calc_key(&self, cursor_pos: usize) -> String {
        format!("{:?}{}", self.canvas.rect(), cursor_pos)
    }

    fn draw(&self, elements: &[crate::menu::MenuElement], cursor_pos: usize) {
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let canvas = &self.canvas;
        let config = &self.app_state.read().unwrap().config.get().scheme();

        canvas.print(
            1,
            0,
            &format!("{} Select to Cd ", SetBackgroundColor(config.label)),
        );
        canvas.print(
            0,
            1,
            &format!(
                "{}{}",
                SetBackgroundColor(config.bar),
                " ".repeat(canvas.rect().width as usize)
            ),
        );

        for (i, element) in elements.iter().enumerate() {
            let cursor = if cursor_pos == i { ">" } else { " " };
            let tag_bg = if cursor_pos == i {
                SetBackgroundColor(config.row_cursor).to_string()
            } else {
                String::new()
            };
            let tag_fg = SetForegroundColor(config.menu_tag);

            canvas.print(
                0,
                i as u16 + 2,
                &format!("{} |{}{} {} ", cursor, tag_bg, tag_fg, element.tag),
            );
        }

        for i in 0..self.canvas.rect().height {
            canvas.print(
                self.canvas.rect().width.saturating_sub(1),
                i,
                &format!(
                    "{}{}{}",
                    SetBackgroundColor(config.bar),
                    SetForegroundColor(config.label),
                    "|"
                ),
            );
        }
    }

    fn reset_size_with_init(&mut self, rect: crate::canvas::Rect) {
        self.canvas = crate::canvas::Canvas::from(rect);
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
            );
            registry.register_key(
                Mode::Menu,
                "j".parse()?,
                CursorDown {
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "k".parse()?,
                CursorUp {
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "J".parse()?,
                CursorToBottom {
                    menu: self.menu.clone(),
                },
            );
            registry.register_key(
                Mode::Menu,
                "K".parse()?,
                CursorToTop {
                    menu: self.menu.clone(),
                },
            );
        }

        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        let rect = *self.menu_rect.read().unwrap();
        if self.menu_canvas.read().unwrap().has_rect_update(rect) {
            self.menu_canvas.write().unwrap().reset_size_with_init(rect);
        }

        let mut menu_canvas = self.menu_canvas.write().unwrap();
        let cursor_pos = self.menu.cursor.current();
        let elements = &self.menu.elements;
        let key = menu_canvas.calc_key(cursor_pos);

        if key != menu_canvas.key {
            menu_canvas.draw(elements, cursor_pos);
            menu_canvas.key = key;
        }

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
