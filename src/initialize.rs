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

    register_key(Normal, "ZZ".into(), command::ExitApp);
    register_key(Normal, "<ESC>".into(), command::ResetView);
    register_key(Normal, "k".into(), command::MoveUp);
    register_key(Normal, "j".into(), command::MoveDown);
    register_key(Normal, "gg".into(), command::MoveTop);
    register_key(Normal, "G".into(), command::MoveBottom);
    register_key(Normal, "gk".into(), command::PageUp);
    register_key(Normal, "gj".into(), command::PageDown);
    register_key(Normal, "h".into(), command::MoveParent);
    register_key(Normal, "l".into(), command::EnterDirOrEdit);
    register_key(Normal, "V".into(), command::VisualSelect);
    register_key(Normal, "M".into(), command::MenuToggle);
    register_key(Normal, "m".into(), command::MenuMove);
    register_key(Normal, "a".into(), command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Normal, "d".into(), command::AskDelete);
    } else {
        register_key(
            Normal,
            "dd".into(),
            command::DeleteFileOrDir {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Normal, "r".into(), command::AskRename);
    register_key(
        Normal,
        "yy".into(),
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Normal, "p".into(), command::AskPaste);
    register_key(Normal, "/".into(), command::Search);
    register_key(Normal, "n".into(), command::SearchNext);

    register_key(Visual, "ZZ".into(), command::ExitApp);
    register_key(Visual, "<ESC>".into(), command::ResetView);
    register_key(Visual, "k".into(), command::MoveUp);
    register_key(Visual, "j".into(), command::MoveDown);
    register_key(Visual, "gg".into(), command::MoveTop);
    register_key(Visual, "G".into(), command::MoveBottom);
    register_key(Visual, "gk".into(), command::PageUp);
    register_key(Visual, "gj".into(), command::PageDown);
    register_key(Visual, "h".into(), command::MoveParent);
    register_key(Visual, "l".into(), command::EnterDirOrEdit);
    register_key(Visual, "V".into(), command::VisualSelect);
    register_key(Visual, "M".into(), command::MenuToggle);
    register_key(Visual, "m".into(), command::MenuMove);
    register_key(Visual, "a".into(), command::AskCreate);
    if crate::config::load().delete.ask {
        register_key(Visual, "d".into(), command::AskDelete);
    } else {
        register_key(
            Visual,
            "d".into(),
            command::DeleteSelected {
                use_tmp: crate::config::load().delete.for_tmp,
                yank_and_native: (
                    crate::config::load().delete.yank,
                    crate::config::load().native_clip,
                ),
            },
        );
    }
    register_key(Visual, "r".into(), command::AskRename);
    register_key(
        Visual,
        "y".into(),
        command::Yank {
            native: crate::config::load().native_clip,
        },
    );
    register_key(Visual, "p".into(), command::AskPaste);
    register_key(Visual, "/".into(), command::Search);
    register_key(Visual, "n".into(), command::SearchNext);

    register_key(Input, "<ESC>".into(), command::DisableInput);
    register_key(Input, "<CR>".into(), command::CompleteInput);
    register_key(Input, "<c-h>".into(), command::InputCursorPrev);
    register_key(Input, "<c-l>".into(), command::InputCursorNext);
    register_key(Input, "<BS>".into(), command::InputDeleteCurrent);
    register_key(Input, "<s-BS>".into(), command::InputDeleteNext);
    register_key(Input, "<SPACE>".into(), command::InputInsert(' '));
    register_key(Input, "!".into(), command::InputInsert('!'));
    register_key(Input, "\"".into(), command::InputInsert('"'));
    register_key(Input, "#".into(), command::InputInsert('#'));
    register_key(Input, "$".into(), command::InputInsert('$'));
    register_key(Input, "%".into(), command::InputInsert('%'));
    register_key(Input, "&".into(), command::InputInsert('&'));
    register_key(Input, "'".into(), command::InputInsert('\''));
    register_key(Input, "(".into(), command::InputInsert('('));
    register_key(Input, ")".into(), command::InputInsert(')'));
    register_key(Input, "*".into(), command::InputInsert('*'));
    register_key(Input, "+".into(), command::InputInsert('+'));
    register_key(Input, ",".into(), command::InputInsert(','));
    register_key(Input, "-".into(), command::InputInsert('-'));
    register_key(Input, ".".into(), command::InputInsert('.'));
    register_key(Input, "/".into(), command::InputInsert('/'));
    register_key(Input, "0".into(), command::InputInsert('0'));
    register_key(Input, "1".into(), command::InputInsert('1'));
    register_key(Input, "2".into(), command::InputInsert('2'));
    register_key(Input, "3".into(), command::InputInsert('3'));
    register_key(Input, "4".into(), command::InputInsert('4'));
    register_key(Input, "5".into(), command::InputInsert('5'));
    register_key(Input, "6".into(), command::InputInsert('6'));
    register_key(Input, "7".into(), command::InputInsert('7'));
    register_key(Input, "8".into(), command::InputInsert('8'));
    register_key(Input, "9".into(), command::InputInsert('9'));
    register_key(Input, ":".into(), command::InputInsert(':'));
    register_key(Input, ";".into(), command::InputInsert(';'));
    register_key(Input, "<lt>".into(), command::InputInsert('<'));
    register_key(Input, "=".into(), command::InputInsert('='));
    register_key(Input, ">".into(), command::InputInsert('>'));
    register_key(Input, "?".into(), command::InputInsert('?'));
    register_key(Input, "@".into(), command::InputInsert('@'));
    register_key(Input, "a".into(), command::InputInsert('a'));
    register_key(Input, "b".into(), command::InputInsert('b'));
    register_key(Input, "c".into(), command::InputInsert('c'));
    register_key(Input, "d".into(), command::InputInsert('d'));
    register_key(Input, "e".into(), command::InputInsert('e'));
    register_key(Input, "f".into(), command::InputInsert('f'));
    register_key(Input, "g".into(), command::InputInsert('g'));
    register_key(Input, "h".into(), command::InputInsert('h'));
    register_key(Input, "i".into(), command::InputInsert('i'));
    register_key(Input, "j".into(), command::InputInsert('j'));
    register_key(Input, "k".into(), command::InputInsert('k'));
    register_key(Input, "l".into(), command::InputInsert('l'));
    register_key(Input, "m".into(), command::InputInsert('m'));
    register_key(Input, "n".into(), command::InputInsert('n'));
    register_key(Input, "o".into(), command::InputInsert('o'));
    register_key(Input, "p".into(), command::InputInsert('p'));
    register_key(Input, "q".into(), command::InputInsert('q'));
    register_key(Input, "r".into(), command::InputInsert('r'));
    register_key(Input, "s".into(), command::InputInsert('s'));
    register_key(Input, "t".into(), command::InputInsert('t'));
    register_key(Input, "u".into(), command::InputInsert('u'));
    register_key(Input, "v".into(), command::InputInsert('v'));
    register_key(Input, "w".into(), command::InputInsert('w'));
    register_key(Input, "x".into(), command::InputInsert('x'));
    register_key(Input, "y".into(), command::InputInsert('y'));
    register_key(Input, "z".into(), command::InputInsert('z'));
    register_key(Input, "A".into(), command::InputInsert('A'));
    register_key(Input, "B".into(), command::InputInsert('B'));
    register_key(Input, "C".into(), command::InputInsert('C'));
    register_key(Input, "D".into(), command::InputInsert('D'));
    register_key(Input, "E".into(), command::InputInsert('E'));
    register_key(Input, "F".into(), command::InputInsert('F'));
    register_key(Input, "G".into(), command::InputInsert('G'));
    register_key(Input, "H".into(), command::InputInsert('H'));
    register_key(Input, "I".into(), command::InputInsert('I'));
    register_key(Input, "J".into(), command::InputInsert('J'));
    register_key(Input, "K".into(), command::InputInsert('K'));
    register_key(Input, "L".into(), command::InputInsert('L'));
    register_key(Input, "M".into(), command::InputInsert('M'));
    register_key(Input, "N".into(), command::InputInsert('N'));
    register_key(Input, "O".into(), command::InputInsert('O'));
    register_key(Input, "P".into(), command::InputInsert('P'));
    register_key(Input, "Q".into(), command::InputInsert('Q'));
    register_key(Input, "R".into(), command::InputInsert('R'));
    register_key(Input, "S".into(), command::InputInsert('S'));
    register_key(Input, "T".into(), command::InputInsert('T'));
    register_key(Input, "U".into(), command::InputInsert('U'));
    register_key(Input, "V".into(), command::InputInsert('V'));
    register_key(Input, "W".into(), command::InputInsert('W'));
    register_key(Input, "X".into(), command::InputInsert('X'));
    register_key(Input, "Y".into(), command::InputInsert('Y'));
    register_key(Input, "Z".into(), command::InputInsert('Z'));
    register_key(Input, "[".into(), command::InputInsert('['));
    register_key(Input, "\\".into(), command::InputInsert('\\'));
    register_key(Input, "]".into(), command::InputInsert(']'));
    register_key(Input, "^".into(), command::InputInsert('^'));
    register_key(Input, "_".into(), command::InputInsert('_'));
    register_key(Input, "`".into(), command::InputInsert('`'));
    register_key(Input, "{".into(), command::InputInsert('{'));
    register_key(Input, "|".into(), command::InputInsert('|'));
    register_key(Input, "}".into(), command::InputInsert('}'));
    register_key(Input, "~".into(), command::InputInsert('~'));

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
