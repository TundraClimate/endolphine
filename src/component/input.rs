use super::{Command, Component};

pub struct Input {
    pub root_state: std::sync::Arc<std::sync::RwLock<super::root::RootState>>,
    pub app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

struct CompleteInput {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CompleteInput {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();

        lock.input.complete_input();
        lock.mode = super::app::Mode::Normal;

        Ok(())
    }
}

struct CancelInput {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for CancelInput {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();

        lock.input.disable();
        lock.mode = super::app::Mode::Normal;

        Ok(())
    }
}

struct InputCursorNext {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputCursorNext {
    fn run(&self) -> Result<(), crate::Error> {
        self.app_state.write().unwrap().input.cursor_right();

        Ok(())
    }
}

struct InputCursorPrev {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputCursorPrev {
    fn run(&self) -> Result<(), crate::Error> {
        self.app_state.write().unwrap().input.cursor_left();

        Ok(())
    }
}

struct InputDeleteCurrent {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputDeleteCurrent {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();
        let input = &mut lock.input;

        input.buffer_pick();

        if input.load_action().as_deref() == Some("Search") {
            // crate::app::sync_grep()
        }

        Ok(())
    }
}

struct InputDeleteNext {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for InputDeleteNext {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();
        let input = &mut lock.input;

        input.buffer_pick_next();

        if input.load_action().as_deref() == Some("Search") {
            // crate::app::sync_grep()
        }

        Ok(())
    }
}

struct InputInsert {
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
    c: char,
}

impl Command for InputInsert {
    fn run(&self) -> Result<(), crate::Error> {
        let mut lock = self.app_state.write().unwrap();
        let input = &mut lock.input;

        input.buffer_insert(self.c);

        if input.load_action().as_deref() == Some("Search") {
            // crate::app::sync_grep()
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
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<ESC>".parse()?,
                CancelInput {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-l>".parse()?,
                InputCursorNext {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<c-h>".parse()?,
                InputCursorPrev {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<BS>".parse()?,
                InputDeleteCurrent {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<s-BS>".parse()?,
                InputDeleteNext {
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Input,
                "<SPACE>".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ' ',
                },
            );
            registry.register_key(
                Mode::Input,
                "!".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '!',
                },
            );
            registry.register_key(
                Mode::Input,
                "\"".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '"',
                },
            );
            registry.register_key(
                Mode::Input,
                "#".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '#',
                },
            );
            registry.register_key(
                Mode::Input,
                "$".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '$',
                },
            );
            registry.register_key(
                Mode::Input,
                "%".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '%',
                },
            );
            registry.register_key(
                Mode::Input,
                "&".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '&',
                },
            );
            registry.register_key(
                Mode::Input,
                "'".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '\'',
                },
            );
            registry.register_key(
                Mode::Input,
                "(".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '(',
                },
            );
            registry.register_key(
                Mode::Input,
                ")".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ')',
                },
            );
            registry.register_key(
                Mode::Input,
                "*".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '*',
                },
            );
            registry.register_key(
                Mode::Input,
                "+".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '+',
                },
            );
            registry.register_key(
                Mode::Input,
                ",".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ',',
                },
            );
            registry.register_key(
                Mode::Input,
                "-".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '-',
                },
            );
            registry.register_key(
                Mode::Input,
                ".".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '.',
                },
            );
            registry.register_key(
                Mode::Input,
                "/".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '/',
                },
            );
            registry.register_key(
                Mode::Input,
                "0".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '0',
                },
            );
            registry.register_key(
                Mode::Input,
                "1".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '1',
                },
            );
            registry.register_key(
                Mode::Input,
                "2".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '2',
                },
            );
            registry.register_key(
                Mode::Input,
                "3".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '3',
                },
            );
            registry.register_key(
                Mode::Input,
                "4".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '4',
                },
            );
            registry.register_key(
                Mode::Input,
                "5".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '5',
                },
            );
            registry.register_key(
                Mode::Input,
                "6".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '6',
                },
            );
            registry.register_key(
                Mode::Input,
                "7".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '7',
                },
            );
            registry.register_key(
                Mode::Input,
                "8".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '8',
                },
            );
            registry.register_key(
                Mode::Input,
                "9".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '9',
                },
            );
            registry.register_key(
                Mode::Input,
                ":".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ':',
                },
            );
            registry.register_key(
                Mode::Input,
                ";".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ';',
                },
            );
            registry.register_key(
                Mode::Input,
                "<lt>".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '<',
                },
            );
            registry.register_key(
                Mode::Input,
                "=".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '=',
                },
            );
            registry.register_key(
                Mode::Input,
                ">".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '>',
                },
            );
            registry.register_key(
                Mode::Input,
                "?".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '?',
                },
            );
            registry.register_key(
                Mode::Input,
                "@".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '@',
                },
            );
            registry.register_key(
                Mode::Input,
                "a".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'a',
                },
            );
            registry.register_key(
                Mode::Input,
                "b".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'b',
                },
            );
            registry.register_key(
                Mode::Input,
                "c".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'c',
                },
            );
            registry.register_key(
                Mode::Input,
                "d".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'd',
                },
            );
            registry.register_key(
                Mode::Input,
                "e".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'e',
                },
            );
            registry.register_key(
                Mode::Input,
                "f".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'f',
                },
            );
            registry.register_key(
                Mode::Input,
                "g".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'g',
                },
            );
            registry.register_key(
                Mode::Input,
                "h".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'h',
                },
            );
            registry.register_key(
                Mode::Input,
                "i".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'i',
                },
            );
            registry.register_key(
                Mode::Input,
                "j".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'j',
                },
            );
            registry.register_key(
                Mode::Input,
                "k".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'k',
                },
            );
            registry.register_key(
                Mode::Input,
                "l".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'l',
                },
            );
            registry.register_key(
                Mode::Input,
                "m".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'm',
                },
            );
            registry.register_key(
                Mode::Input,
                "n".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'n',
                },
            );
            registry.register_key(
                Mode::Input,
                "o".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'o',
                },
            );
            registry.register_key(
                Mode::Input,
                "p".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'p',
                },
            );
            registry.register_key(
                Mode::Input,
                "q".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'q',
                },
            );
            registry.register_key(
                Mode::Input,
                "r".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'r',
                },
            );
            registry.register_key(
                Mode::Input,
                "s".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 's',
                },
            );
            registry.register_key(
                Mode::Input,
                "t".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 't',
                },
            );
            registry.register_key(
                Mode::Input,
                "u".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'u',
                },
            );
            registry.register_key(
                Mode::Input,
                "v".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'v',
                },
            );
            registry.register_key(
                Mode::Input,
                "w".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'w',
                },
            );
            registry.register_key(
                Mode::Input,
                "x".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'x',
                },
            );
            registry.register_key(
                Mode::Input,
                "y".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'y',
                },
            );
            registry.register_key(
                Mode::Input,
                "z".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'z',
                },
            );
            registry.register_key(
                Mode::Input,
                "A".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'A',
                },
            );
            registry.register_key(
                Mode::Input,
                "B".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'B',
                },
            );
            registry.register_key(
                Mode::Input,
                "C".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'C',
                },
            );
            registry.register_key(
                Mode::Input,
                "D".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'D',
                },
            );
            registry.register_key(
                Mode::Input,
                "E".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'E',
                },
            );
            registry.register_key(
                Mode::Input,
                "F".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'F',
                },
            );
            registry.register_key(
                Mode::Input,
                "G".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'G',
                },
            );
            registry.register_key(
                Mode::Input,
                "H".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'H',
                },
            );
            registry.register_key(
                Mode::Input,
                "I".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'I',
                },
            );
            registry.register_key(
                Mode::Input,
                "J".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'J',
                },
            );
            registry.register_key(
                Mode::Input,
                "K".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'K',
                },
            );
            registry.register_key(
                Mode::Input,
                "L".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'L',
                },
            );
            registry.register_key(
                Mode::Input,
                "M".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'M',
                },
            );
            registry.register_key(
                Mode::Input,
                "N".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'N',
                },
            );
            registry.register_key(
                Mode::Input,
                "O".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'O',
                },
            );
            registry.register_key(
                Mode::Input,
                "P".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'P',
                },
            );
            registry.register_key(
                Mode::Input,
                "Q".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'Q',
                },
            );
            registry.register_key(
                Mode::Input,
                "R".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'R',
                },
            );
            registry.register_key(
                Mode::Input,
                "S".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'S',
                },
            );
            registry.register_key(
                Mode::Input,
                "T".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'T',
                },
            );
            registry.register_key(
                Mode::Input,
                "U".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'U',
                },
            );
            registry.register_key(
                Mode::Input,
                "V".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'V',
                },
            );
            registry.register_key(
                Mode::Input,
                "W".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'W',
                },
            );
            registry.register_key(
                Mode::Input,
                "X".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'X',
                },
            );
            registry.register_key(
                Mode::Input,
                "Y".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'Y',
                },
            );
            registry.register_key(
                Mode::Input,
                "Z".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: 'Z',
                },
            );
            registry.register_key(
                Mode::Input,
                "[".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '[',
                },
            );
            registry.register_key(
                Mode::Input,
                "\\".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '\\',
                },
            );
            registry.register_key(
                Mode::Input,
                "]".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: ']',
                },
            );
            registry.register_key(
                Mode::Input,
                "^".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '^',
                },
            );
            registry.register_key(
                Mode::Input,
                "_".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '_',
                },
            );
            registry.register_key(
                Mode::Input,
                "`".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '`',
                },
            );
            registry.register_key(
                Mode::Input,
                "{".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '{',
                },
            );
            registry.register_key(
                Mode::Input,
                "|".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '|',
                },
            );
            registry.register_key(
                Mode::Input,
                "}".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '}',
                },
            );
            registry.register_key(
                Mode::Input,
                "~".parse()?,
                InputInsert {
                    app_state: self.app_state.clone(),
                    c: '~',
                },
            );
        }

        Ok(())
    }
}
