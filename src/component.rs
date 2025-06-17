mod app;
mod body;
mod event_reader;
mod input;
mod input_handler;
mod key_handler;
mod root;
mod sidemenu;

pub trait Component: Send + Sync {
    fn on_init(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn on_resize(&self, _size: (u16, u16)) -> Result<(), crate::Error> {
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct CommandContext {
    prenum: Option<usize>,
}

trait Command: Send + Sync {
    fn run(&self, ctx: CommandContext) -> Result<(), crate::Error>;
}

pub fn components() -> Box<dyn Component> {
    use app::App;
    use body::Body;
    use event_reader::EventReader;
    use input::Input;
    use input_handler::InputHandler;
    use key_handler::KeyHandler;
    use root::Root;
    use sidemenu::SideMenu;

    Box::new(Root::with_state(|root_state, base_rect| {
        vec![
            Box::new(EventReader {
                root_state: root_state.clone(),
            }),
            Box::new(App::with_state(
                root_state.clone(),
                base_rect.clone(),
                |app_state, (menu_rect, body_rect)| {
                    vec![
                        Box::new(KeyHandler {
                            root_state: root_state.clone(),
                            app_state: app_state.clone(),
                        }),
                        Box::new(Body::with_state(
                            app_state.clone(),
                            root_state.clone(),
                            body_rect.clone(),
                            |body_state| {
                                vec![
                                    Box::new(Input {
                                        root_state: root_state.clone(),
                                        body_state: body_state.clone(),
                                        app_state: app_state.clone(),
                                    }),
                                    Box::new(InputHandler {
                                        root_state: root_state.clone(),
                                        body_state: body_state.clone(),
                                        app_state: app_state.clone(),
                                    }),
                                ]
                            },
                        )),
                        Box::new(SideMenu::new(
                            root_state.clone(),
                            app_state.clone(),
                            menu_rect.clone(),
                            std::sync::Arc::new(crate::menu::Menu::default()),
                        )),
                    ]
                },
            )),
        ]
    }))
}
