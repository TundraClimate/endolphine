mod app;
mod body;
mod input;
mod input_handler;
mod key_handler;
mod key_reader;
mod root;

pub trait Component: Send + Sync {
    fn on_init(&self) -> Result<(), crate::Error> {
        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        Ok(())
    }
}

struct CommandContext {
    prenum: Option<usize>,
}

trait Command: Send + Sync {
    fn run(&self, ctx: CommandContext) -> Result<(), crate::Error>;
}

pub fn components() -> Box<dyn Component> {
    use app::App;
    use body::Body;
    use input::Input;
    use input_handler::InputHandler;
    use key_handler::KeyHandler;
    use key_reader::KeyReader;
    use root::Root;

    Box::new(Root::with_state(|root_state| {
        vec![
            Box::new(KeyReader {
                root_state: root_state.clone(),
            }),
            Box::new(App::with_state(root_state.clone(), |app_state| {
                vec![
                    Box::new(KeyHandler {
                        root_state: root_state.clone(),
                        app_state: app_state.clone(),
                    }),
                    Box::new(Body::with_state(
                        app_state.clone(),
                        root_state.clone(),
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
                ]
            })),
        ]
    }))
}
