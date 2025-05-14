use super::{Command, Component};

pub struct Input {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

struct CompleteInput {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CompleteInput {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();

        self.body_state.write().unwrap().input.complete_input();
        lock.mode = super::app::Mode::Normal;

        Ok(())
    }
}

struct CancelInput {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CancelInput {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();

        self.body_state.write().unwrap().input.disable();
        lock.mode = super::app::Mode::Normal;

        Ok(())
    }
}

struct InputCursorNext {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
}

impl Command for InputCursorNext {
    fn run(&self) -> Result<(), crate::Error> {
        self.body_state.write().unwrap().input.cursor_right();

        Ok(())
    }
}

struct InputCursorPrev {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
}

impl Command for InputCursorPrev {
    fn run(&self) -> Result<(), crate::Error> {
        self.body_state.write().unwrap().input.cursor_left();

        Ok(())
    }
}

struct InputDeleteCurrent {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
}

impl Command for InputDeleteCurrent {
    fn run(&self) -> Result<(), crate::Error> {
        let mut buf: Option<String> = None;

        {
            let mut lock = self.body_state.write().unwrap();

            let input = &mut lock.input;

            input.buffer_pick();

            if input.load_action().as_deref() == Some("Search") {
                if let Some(buffer) = input.buffer_load() {
                    buf = Some(buffer.clone());
                }
            }
        }

        if let Some(buf) = buf {
            self.body_state
                .write()
                .unwrap()
                .grep
                .set_with_strip_preslash(&buf);
        }

        Ok(())
    }
}

struct InputDeleteNext {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
}

impl Command for InputDeleteNext {
    fn run(&self) -> Result<(), crate::Error> {
        let mut buf: Option<String> = None;

        {
            let mut lock = self.body_state.write().unwrap();
            let input = &mut lock.input;

            input.buffer_pick_next();

            if input.load_action().as_deref() == Some("Search") {
                if let Some(buffer) = input.buffer_load() {
                    buf = Some(buffer.clone());
                }
            }
        }

        if let Some(buf) = buf {
            self.body_state
                .write()
                .unwrap()
                .grep
                .set_with_strip_preslash(&buf);
        }

        Ok(())
    }
}

struct InputInsert {
    body_state: std::sync::Arc<std::sync::RwLock<super::body::BodyState>>,
    c: char,
}

impl Command for InputInsert {
    fn run(&self) -> Result<(), crate::Error> {
        let mut buf: Option<String> = None;

        {
            let mut lock = self.body_state.write().unwrap();
            let input = &mut lock.input;

            input.buffer_insert(self.c);

            if input.load_action().as_deref() == Some("Search") {
                if let Some(buffer) = input.buffer_load() {
                    buf = Some(buffer.clone());
                }
            }
        }

        if let Some(buf) = buf {
            self.body_state
                .write()
                .unwrap()
                .grep
                .set_with_strip_preslash(&buf);
        }

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
                    body_state: self.body_state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<ESC>".parse()?,
                CancelInput {
                    body_state: self.body_state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-l>".parse()?,
                InputCursorNext {
                    body_state: self.body_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-h>".parse()?,
                InputCursorPrev {
                    body_state: self.body_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<BS>".parse()?,
                InputDeleteCurrent {
                    body_state: self.body_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<s-BS>".parse()?,
                InputDeleteNext {
                    body_state: self.body_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<SPACE>".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ' ',
                },
            );
            registry.register_key(
                Mode::Input,
                "!".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '!',
                },
            );
            registry.register_key(
                Mode::Input,
                "\"".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '"',
                },
            );
            registry.register_key(
                Mode::Input,
                "#".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '#',
                },
            );
            registry.register_key(
                Mode::Input,
                "$".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '$',
                },
            );
            registry.register_key(
                Mode::Input,
                "%".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '%',
                },
            );
            registry.register_key(
                Mode::Input,
                "&".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '&',
                },
            );
            registry.register_key(
                Mode::Input,
                "'".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '\'',
                },
            );
            registry.register_key(
                Mode::Input,
                "(".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '(',
                },
            );
            registry.register_key(
                Mode::Input,
                ")".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ')',
                },
            );
            registry.register_key(
                Mode::Input,
                "*".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '*',
                },
            );
            registry.register_key(
                Mode::Input,
                "+".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '+',
                },
            );
            registry.register_key(
                Mode::Input,
                ",".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ',',
                },
            );
            registry.register_key(
                Mode::Input,
                "-".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '-',
                },
            );
            registry.register_key(
                Mode::Input,
                ".".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '.',
                },
            );
            registry.register_key(
                Mode::Input,
                "/".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '/',
                },
            );
            registry.register_key(
                Mode::Input,
                "0".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '0',
                },
            );
            registry.register_key(
                Mode::Input,
                "1".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '1',
                },
            );
            registry.register_key(
                Mode::Input,
                "2".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '2',
                },
            );
            registry.register_key(
                Mode::Input,
                "3".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '3',
                },
            );
            registry.register_key(
                Mode::Input,
                "4".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '4',
                },
            );
            registry.register_key(
                Mode::Input,
                "5".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '5',
                },
            );
            registry.register_key(
                Mode::Input,
                "6".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '6',
                },
            );
            registry.register_key(
                Mode::Input,
                "7".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '7',
                },
            );
            registry.register_key(
                Mode::Input,
                "8".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '8',
                },
            );
            registry.register_key(
                Mode::Input,
                "9".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '9',
                },
            );
            registry.register_key(
                Mode::Input,
                ":".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ':',
                },
            );
            registry.register_key(
                Mode::Input,
                ";".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ';',
                },
            );
            registry.register_key(
                Mode::Input,
                "<lt>".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '<',
                },
            );
            registry.register_key(
                Mode::Input,
                "=".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '=',
                },
            );
            registry.register_key(
                Mode::Input,
                ">".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '>',
                },
            );
            registry.register_key(
                Mode::Input,
                "?".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '?',
                },
            );
            registry.register_key(
                Mode::Input,
                "@".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '@',
                },
            );
            registry.register_key(
                Mode::Input,
                "a".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'a',
                },
            );
            registry.register_key(
                Mode::Input,
                "b".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'b',
                },
            );
            registry.register_key(
                Mode::Input,
                "c".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'c',
                },
            );
            registry.register_key(
                Mode::Input,
                "d".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'd',
                },
            );
            registry.register_key(
                Mode::Input,
                "e".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'e',
                },
            );
            registry.register_key(
                Mode::Input,
                "f".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'f',
                },
            );
            registry.register_key(
                Mode::Input,
                "g".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'g',
                },
            );
            registry.register_key(
                Mode::Input,
                "h".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'h',
                },
            );
            registry.register_key(
                Mode::Input,
                "i".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'i',
                },
            );
            registry.register_key(
                Mode::Input,
                "j".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'j',
                },
            );
            registry.register_key(
                Mode::Input,
                "k".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'k',
                },
            );
            registry.register_key(
                Mode::Input,
                "l".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'l',
                },
            );
            registry.register_key(
                Mode::Input,
                "m".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'm',
                },
            );
            registry.register_key(
                Mode::Input,
                "n".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'n',
                },
            );
            registry.register_key(
                Mode::Input,
                "o".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'o',
                },
            );
            registry.register_key(
                Mode::Input,
                "p".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'p',
                },
            );
            registry.register_key(
                Mode::Input,
                "q".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'q',
                },
            );
            registry.register_key(
                Mode::Input,
                "r".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'r',
                },
            );
            registry.register_key(
                Mode::Input,
                "s".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 's',
                },
            );
            registry.register_key(
                Mode::Input,
                "t".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 't',
                },
            );
            registry.register_key(
                Mode::Input,
                "u".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'u',
                },
            );
            registry.register_key(
                Mode::Input,
                "v".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'v',
                },
            );
            registry.register_key(
                Mode::Input,
                "w".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'w',
                },
            );
            registry.register_key(
                Mode::Input,
                "x".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'x',
                },
            );
            registry.register_key(
                Mode::Input,
                "y".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'y',
                },
            );
            registry.register_key(
                Mode::Input,
                "z".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'z',
                },
            );
            registry.register_key(
                Mode::Input,
                "A".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'A',
                },
            );
            registry.register_key(
                Mode::Input,
                "B".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'B',
                },
            );
            registry.register_key(
                Mode::Input,
                "C".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'C',
                },
            );
            registry.register_key(
                Mode::Input,
                "D".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'D',
                },
            );
            registry.register_key(
                Mode::Input,
                "E".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'E',
                },
            );
            registry.register_key(
                Mode::Input,
                "F".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'F',
                },
            );
            registry.register_key(
                Mode::Input,
                "G".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'G',
                },
            );
            registry.register_key(
                Mode::Input,
                "H".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'H',
                },
            );
            registry.register_key(
                Mode::Input,
                "I".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'I',
                },
            );
            registry.register_key(
                Mode::Input,
                "J".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'J',
                },
            );
            registry.register_key(
                Mode::Input,
                "K".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'K',
                },
            );
            registry.register_key(
                Mode::Input,
                "L".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'L',
                },
            );
            registry.register_key(
                Mode::Input,
                "M".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'M',
                },
            );
            registry.register_key(
                Mode::Input,
                "N".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'N',
                },
            );
            registry.register_key(
                Mode::Input,
                "O".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'O',
                },
            );
            registry.register_key(
                Mode::Input,
                "P".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'P',
                },
            );
            registry.register_key(
                Mode::Input,
                "Q".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'Q',
                },
            );
            registry.register_key(
                Mode::Input,
                "R".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'R',
                },
            );
            registry.register_key(
                Mode::Input,
                "S".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'S',
                },
            );
            registry.register_key(
                Mode::Input,
                "T".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'T',
                },
            );
            registry.register_key(
                Mode::Input,
                "U".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'U',
                },
            );
            registry.register_key(
                Mode::Input,
                "V".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'V',
                },
            );
            registry.register_key(
                Mode::Input,
                "W".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'W',
                },
            );
            registry.register_key(
                Mode::Input,
                "X".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'X',
                },
            );
            registry.register_key(
                Mode::Input,
                "Y".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'Y',
                },
            );
            registry.register_key(
                Mode::Input,
                "Z".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: 'Z',
                },
            );
            registry.register_key(
                Mode::Input,
                "[".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '[',
                },
            );
            registry.register_key(
                Mode::Input,
                "\\".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '\\',
                },
            );
            registry.register_key(
                Mode::Input,
                "]".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: ']',
                },
            );
            registry.register_key(
                Mode::Input,
                "^".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '^',
                },
            );
            registry.register_key(
                Mode::Input,
                "_".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '_',
                },
            );
            registry.register_key(
                Mode::Input,
                "`".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '`',
                },
            );
            registry.register_key(
                Mode::Input,
                "{".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '{',
                },
            );
            registry.register_key(
                Mode::Input,
                "|".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '|',
                },
            );
            registry.register_key(
                Mode::Input,
                "}".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '}',
                },
            );
            registry.register_key(
                Mode::Input,
                "~".parse()?,
                InputInsert {
                    body_state: self.body_state.clone(),
                    c: '~',
                },
            );
        }

        Ok(())
    }
}
