pub fn application(path: &std::path::Path) -> Result<(), crate::Error> {
    let path = path.canonicalize().map_err(|e| {
        crate::sys_log!("e", "Couldn't get the canonicalized path");
        crate::Error::FilesystemError(e.kind().to_string())
    })?;

    crate::app::set_path(&path);

    let c = crate::misc::child_files_len(&path);
    crate::cursor::load().resize(c);

    if crate::config::load().delete.for_tmp {
        let tmp_path = std::path::Path::new("/tmp").join("endolphine");
        if !tmp_path.exists() {
            std::fs::create_dir_all(tmp_path).map_err(|e| {
                crate::sys_log!("e", "Couldn't create the \"/tmp/\"");
                crate::Error::FilesystemError(e.kind().to_string())
            })?;
        }
    }

    let log_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
        .join(".local")
        .join("share")
        .join("endolphine")
        .join("log");

    if !log_path.exists() {
        std::fs::create_dir_all(log_path).map_err(|e| {
            crate::sys_log!("e", "Couldn't create the log directory");
            crate::Error::FilesystemError(e.kind().to_string())
        })?;
    }

    Ok(())
}

pub fn config() -> Result<(), crate::Error> {
    let conf_path = crate::config::file_path();
    if let Some(conf_path) = conf_path {
        if !conf_path.exists() {
            let parent = crate::misc::parent(&conf_path);

            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration dir");
                    crate::Error::FilesystemError(e.kind().to_string())
                })?;
            }

            let config_default = toml::to_string_pretty(&crate::config::Config::default())
                .map_err(|e| {
                    crate::sys_log!("e", "Couldn't generate the default configration");
                    crate::Error::TomlParseFailed(e.to_string())
                })?;

            if !conf_path.exists() {
                std::fs::write(&conf_path, config_default).map_err(|e| {
                    crate::sys_log!("e", "Couldn't create the configration file");
                    crate::Error::FilesystemError(e.kind().to_string())
                })?;
            }
        }
    }

    Ok(())
}

pub fn keymap() -> Result<(), crate::Error> {
    use crate::app::AppMode::{Input, Normal, Visual};
    use crate::command;
    use crate::config::register_key;

    register_key(Normal, "ZZ".parse()?, command::ExitApp);
    register_key(Normal, "<ESC>".parse()?, command::ResetView);
    register_key(Normal, "k".parse()?, command::MoveUp);
    register_key(Normal, "j".parse()?, command::MoveDown);
    register_key(Normal, "gg".parse()?, command::MoveTop);
    register_key(Normal, "G".parse()?, command::MoveBottom);
    register_key(Normal, "gk".parse()?, command::PageUp);
    register_key(Normal, "gj".parse()?, command::PageDown);
    register_key(Normal, "h".parse()?, command::MoveParent);
    register_key(Normal, "l".parse()?, command::EnterDirOrEdit);
    register_key(Normal, "V".parse()?, command::VisualSelect);
    register_key(Normal, "M".parse()?, command::MenuToggle);
    register_key(Normal, "m".parse()?, command::MenuMove);
    register_key(Normal, "a".parse()?, command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Normal, "d".parse()?, command::AskDelete);
    } else {
        register_key(
            Normal,
            "dd".parse()?,
            command::DeleteFileOrDir {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Normal, "r".parse()?, command::AskRename);
    register_key(
        Normal,
        "yy".parse()?,
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Normal, "p".parse()?, command::AskPaste);
    register_key(Normal, "/".parse()?, command::Search);
    register_key(Normal, "n".parse()?, command::SearchNext);

    register_key(Visual, "ZZ".parse()?, command::ExitApp);
    register_key(Visual, "<ESC>".parse()?, command::ResetView);
    register_key(Visual, "k".parse()?, command::MoveUp);
    register_key(Visual, "j".parse()?, command::MoveDown);
    register_key(Visual, "gg".parse()?, command::MoveTop);
    register_key(Visual, "G".parse()?, command::MoveBottom);
    register_key(Visual, "gk".parse()?, command::PageUp);
    register_key(Visual, "gj".parse()?, command::PageDown);
    register_key(Visual, "h".parse()?, command::MoveParent);
    register_key(Visual, "l".parse()?, command::EnterDirOrEdit);
    register_key(Visual, "V".parse()?, command::VisualSelect);
    register_key(Visual, "M".parse()?, command::MenuToggle);
    register_key(Visual, "m".parse()?, command::MenuMove);
    register_key(Visual, "a".parse()?, command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Visual, "d".parse()?, command::AskDelete);
    } else {
        register_key(
            Visual,
            "d".parse()?,
            command::DeleteSelected {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Visual, "r".parse()?, command::AskRename);
    register_key(
        Visual,
        "y".parse()?,
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Visual, "p".parse()?, command::AskPaste);
    register_key(Visual, "/".parse()?, command::Search);
    register_key(Visual, "n".parse()?, command::SearchNext);

    register_key(Input, "<ESC>".parse()?, command::DisableInput);
    register_key(Input, "<CR>".parse()?, command::CompleteInput);
    register_key(Input, "<c-h>".parse()?, command::InputCursorPrev);
    register_key(Input, "<c-l>".parse()?, command::InputCursorNext);
    register_key(Input, "<BS>".parse()?, command::InputDeleteCurrent);
    register_key(Input, "<s-BS>".parse()?, command::InputDeleteNext);
    register_key(Input, "<SPACE>".parse()?, command::InputInsert(' '));
    register_key(Input, "!".parse()?, command::InputInsert('!'));
    register_key(Input, "\"".parse()?, command::InputInsert('"'));
    register_key(Input, "#".parse()?, command::InputInsert('#'));
    register_key(Input, "$".parse()?, command::InputInsert('$'));
    register_key(Input, "%".parse()?, command::InputInsert('%'));
    register_key(Input, "&".parse()?, command::InputInsert('&'));
    register_key(Input, "'".parse()?, command::InputInsert('\''));
    register_key(Input, "(".parse()?, command::InputInsert('('));
    register_key(Input, ")".parse()?, command::InputInsert(')'));
    register_key(Input, "*".parse()?, command::InputInsert('*'));
    register_key(Input, "+".parse()?, command::InputInsert('+'));
    register_key(Input, ",".parse()?, command::InputInsert(','));
    register_key(Input, "-".parse()?, command::InputInsert('-'));
    register_key(Input, ".".parse()?, command::InputInsert('.'));
    register_key(Input, "/".parse()?, command::InputInsert('/'));
    register_key(Input, "0".parse()?, command::InputInsert('0'));
    register_key(Input, "1".parse()?, command::InputInsert('1'));
    register_key(Input, "2".parse()?, command::InputInsert('2'));
    register_key(Input, "3".parse()?, command::InputInsert('3'));
    register_key(Input, "4".parse()?, command::InputInsert('4'));
    register_key(Input, "5".parse()?, command::InputInsert('5'));
    register_key(Input, "6".parse()?, command::InputInsert('6'));
    register_key(Input, "7".parse()?, command::InputInsert('7'));
    register_key(Input, "8".parse()?, command::InputInsert('8'));
    register_key(Input, "9".parse()?, command::InputInsert('9'));
    register_key(Input, ":".parse()?, command::InputInsert(':'));
    register_key(Input, ";".parse()?, command::InputInsert(';'));
    register_key(Input, "<lt>".parse()?, command::InputInsert('<'));
    register_key(Input, "=".parse()?, command::InputInsert('='));
    register_key(Input, ">".parse()?, command::InputInsert('>'));
    register_key(Input, "?".parse()?, command::InputInsert('?'));
    register_key(Input, "@".parse()?, command::InputInsert('@'));
    register_key(Input, "a".parse()?, command::InputInsert('a'));
    register_key(Input, "b".parse()?, command::InputInsert('b'));
    register_key(Input, "c".parse()?, command::InputInsert('c'));
    register_key(Input, "d".parse()?, command::InputInsert('d'));
    register_key(Input, "e".parse()?, command::InputInsert('e'));
    register_key(Input, "f".parse()?, command::InputInsert('f'));
    register_key(Input, "g".parse()?, command::InputInsert('g'));
    register_key(Input, "h".parse()?, command::InputInsert('h'));
    register_key(Input, "i".parse()?, command::InputInsert('i'));
    register_key(Input, "j".parse()?, command::InputInsert('j'));
    register_key(Input, "k".parse()?, command::InputInsert('k'));
    register_key(Input, "l".parse()?, command::InputInsert('l'));
    register_key(Input, "m".parse()?, command::InputInsert('m'));
    register_key(Input, "n".parse()?, command::InputInsert('n'));
    register_key(Input, "o".parse()?, command::InputInsert('o'));
    register_key(Input, "p".parse()?, command::InputInsert('p'));
    register_key(Input, "q".parse()?, command::InputInsert('q'));
    register_key(Input, "r".parse()?, command::InputInsert('r'));
    register_key(Input, "s".parse()?, command::InputInsert('s'));
    register_key(Input, "t".parse()?, command::InputInsert('t'));
    register_key(Input, "u".parse()?, command::InputInsert('u'));
    register_key(Input, "v".parse()?, command::InputInsert('v'));
    register_key(Input, "w".parse()?, command::InputInsert('w'));
    register_key(Input, "x".parse()?, command::InputInsert('x'));
    register_key(Input, "y".parse()?, command::InputInsert('y'));
    register_key(Input, "z".parse()?, command::InputInsert('z'));
    register_key(Input, "A".parse()?, command::InputInsert('A'));
    register_key(Input, "B".parse()?, command::InputInsert('B'));
    register_key(Input, "C".parse()?, command::InputInsert('C'));
    register_key(Input, "D".parse()?, command::InputInsert('D'));
    register_key(Input, "E".parse()?, command::InputInsert('E'));
    register_key(Input, "F".parse()?, command::InputInsert('F'));
    register_key(Input, "G".parse()?, command::InputInsert('G'));
    register_key(Input, "H".parse()?, command::InputInsert('H'));
    register_key(Input, "I".parse()?, command::InputInsert('I'));
    register_key(Input, "J".parse()?, command::InputInsert('J'));
    register_key(Input, "K".parse()?, command::InputInsert('K'));
    register_key(Input, "L".parse()?, command::InputInsert('L'));
    register_key(Input, "M".parse()?, command::InputInsert('M'));
    register_key(Input, "N".parse()?, command::InputInsert('N'));
    register_key(Input, "O".parse()?, command::InputInsert('O'));
    register_key(Input, "P".parse()?, command::InputInsert('P'));
    register_key(Input, "Q".parse()?, command::InputInsert('Q'));
    register_key(Input, "R".parse()?, command::InputInsert('R'));
    register_key(Input, "S".parse()?, command::InputInsert('S'));
    register_key(Input, "T".parse()?, command::InputInsert('T'));
    register_key(Input, "U".parse()?, command::InputInsert('U'));
    register_key(Input, "V".parse()?, command::InputInsert('V'));
    register_key(Input, "W".parse()?, command::InputInsert('W'));
    register_key(Input, "X".parse()?, command::InputInsert('X'));
    register_key(Input, "Y".parse()?, command::InputInsert('Y'));
    register_key(Input, "Z".parse()?, command::InputInsert('Z'));
    register_key(Input, "[".parse()?, command::InputInsert('['));
    register_key(Input, "\\".parse()?, command::InputInsert('\\'));
    register_key(Input, "]".parse()?, command::InputInsert(']'));
    register_key(Input, "^".parse()?, command::InputInsert('^'));
    register_key(Input, "_".parse()?, command::InputInsert('_'));
    register_key(Input, "`".parse()?, command::InputInsert('`'));
    register_key(Input, "{".parse()?, command::InputInsert('{'));
    register_key(Input, "|".parse()?, command::InputInsert('|'));
    register_key(Input, "}".parse()?, command::InputInsert('}'));
    register_key(Input, "~".parse()?, command::InputInsert('~'));

    if let Some(ref define) = crate::config::load().keymap {
        if let Some(normal) = define.normal_key_map() {
            normal
                .into_iter()
                .for_each(|(from, to)| register_key(Normal, from, to))
        }

        if let Some(visual) = define.visual_key_map() {
            visual
                .into_iter()
                .for_each(|(from, to)| register_key(Visual, from, to))
        }

        if let Some(input) = define.input_key_map() {
            input
                .into_iter()
                .for_each(|(from, to)| register_key(Input, from, to))
        }
    }

    Ok(())
}
