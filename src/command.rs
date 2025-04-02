use crate::{app, canvas, clipboard, config, cursor, input, menu, misc};

pub trait Command {
    fn run(&self) -> Result<(), app::Error>;
}

fn move_current_dir(path: &std::path::Path) {
    let cursor = cursor::load();

    cursor::disable_selection();
    app::set_path(path);

    crate::sys_log!("i", "Change the open directory: {}", path.to_string_lossy());

    canvas::cache_clear();
    app::grep_update(|m| m.clear());

    cursor.resize(misc::child_files_len(path));
    cursor.reset();
}

pub struct ExitApp;

impl Command for ExitApp {
    fn run(&self) -> Result<(), app::Error> {
        app::disable_tui()?;

        crate::sys_log!("i", "Endolphine close successfully");

        std::process::exit(0)
    }
}

pub struct ResetView;

impl Command for ResetView {
    fn run(&self) -> Result<(), app::Error> {
        cursor::disable_selection();

        Ok(())
    }
}

pub struct Move(pub isize);

impl Command for Move {
    fn run(&self) -> Result<(), app::Error> {
        let cursor = cursor::captured();

        cursor.shift(self.0);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor.current());
        }

        Ok(())
    }
}

pub struct MoveParent;

impl Command for MoveParent {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let path = app::get_path();

        if path == std::path::Path::new("/") {
            return Ok(());
        }

        let parent = misc::parent(&path);
        let cursor = cursor::load();
        let child_files = misc::sorted_child_files(&path);

        if let Some(target_path) = child_files.get(cursor.current()) {
            let mut cur = cursor.cache.write().unwrap();
            cur.wrap_node(target_path);
        }

        move_current_dir(&parent);

        let child_files = misc::sorted_child_files(&parent);

        if let Some(pos) = child_files.into_iter().position(|p| p == path) {
            cursor.shift(pos as isize);
        }

        Ok(())
    }
}

pub struct EnterDirOrEdit;

impl Command for EnterDirOrEdit {
    fn run(&self) -> Result<(), app::Error> {
        let cursor = cursor::captured();
        let menu = menu::refs();

        if menu.is_enabled() {
            if let Some(element) = menu.elements.get(cursor.current()) {
                let path = &element.path;

                if !path.is_dir() {
                    crate::sys_log!("w", "Found the invalid Shortcut in MENU: {}", element.tag);
                    crate::log!("\"{}\" is not Directory", element.tag);

                    return Ok(());
                }

                move_current_dir(path);
                menu.toggle_enable();
                cursor::load().cache.write().unwrap().reset();
            }

            return Ok(());
        }

        let path = app::get_path();
        let child_files = misc::sorted_child_files(&path);

        if child_files.is_empty() {
            return Ok(());
        }

        let Some(target_path) = child_files.get(cursor.current()) else {
            return Ok(());
        };

        if target_path.is_dir() {
            let child_files = misc::sorted_child_files(target_path);

            move_current_dir(target_path);

            let mut cache = cursor.cache.write().unwrap();

            if let Some(pos) = child_files.iter().position(|e| cache.inner_equal(e)) {
                cursor.shift(pos as isize);
                cache.unwrap_surface();
            } else {
                cache.reset();
            }
        } else if target_path.is_file() {
            let mut cmd = config::load().editor.clone();
            let mut in_term = true;

            if let Some(extension) = target_path.extension().map(|e| e.to_string_lossy()) {
                if let Some(opts) = config::load().opener.corresponding_with(&extension) {
                    cmd = opts.cmd;
                    in_term = opts.in_term.unwrap_or(true);

                    crate::sys_log!("i", "Override open command: {}", cmd.join(" "));
                }
            }

            let Some((cmd, args)) = cmd.split_first() else {
                crate::sys_log!("w", "Invalid config: open command is empty");
                crate::log!("Invalid config: editor or opener");

                return Ok(());
            };

            if in_term {
                app::disable_tui()?;
            }

            crate::sys_log!(
                "i",
                "Open file with {}: {}",
                cmd,
                target_path.to_string_lossy()
            );

            std::process::Command::new(cmd)
                .args(args)
                .arg(target_path)
                .status()
                .map_err(|e| {
                    app::Error::CommandExecutionFailed(cmd.to_owned(), e.kind().to_string())
                })?;

            if in_term {
                app::enable_tui()?;
                canvas::cache_clear();
            }
        }

        Ok(())
    }
}

pub struct VisualSelect;

impl Command for VisualSelect {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::toggle_selection(cursor::load().current());

        Ok(())
    }
}

pub struct MenuToggle;

impl Command for MenuToggle {
    fn run(&self) -> Result<(), app::Error> {
        if !menu::is_opened() || menu::refs().is_enabled() {
            menu::refs().toggle_enable();
        }

        menu::toggle_open();
        canvas::cache_clear();

        Ok(())
    }
}

pub struct MenuMove;

impl Command for MenuMove {
    fn run(&self) -> Result<(), app::Error> {
        if !menu::is_opened() {
            menu::toggle_open();
        }

        menu::refs().toggle_enable();
        canvas::cache_clear();

        Ok(())
    }
}

pub struct CreateNew;

impl Command for CreateNew {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();
        input::use_f_mut(|i| i.enable("", Some("AddNewFileOrDirectory".into())));
        crate::sys_log!("i", "Called command: AddNewFileOrDirectory");
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");

        Ok(())
    }
}

pub struct Delete;

impl Command for Delete {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        if cursor::is_selection() {
            let selected_files = misc::sorted_child_files(&app::get_path())
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
                .collect::<Vec<_>>();

            input::use_f_mut(|i| i.enable("", Some("RmSelected".into())));
            crate::sys_log!("i", "Called command: RmSelected");
            crate::log!("Delete {} items ? (y/Y)", selected_files.len());

            return Ok(());
        }

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            input::use_f_mut(|i| i.enable("", Some("RmFileOrDirectory".into())));
            crate::sys_log!("i", "Called command: RmFileOrDirectory");
            crate::log!("Delete \"{}\" ? (y/Y)", misc::file_name(under_cursor_file));
        }

        Ok(())
    }
}

pub struct Rename;

impl Command for Rename {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        let cursor = cursor::load();

        cursor::disable_selection();

        if let Some(under_cursor_file) =
            misc::sorted_child_files(&app::get_path()).get(cursor.current())
        {
            let name = misc::file_name(under_cursor_file);

            input::use_f_mut(|i| i.enable(name, Some("Rename".into())));
            crate::sys_log!("i", "Called command: Rename");
            crate::log!("Enter new name for \"{}\"", name);
        }

        Ok(())
    }
}

pub struct Yank {
    pub native: bool,
}

impl Command for Yank {
    fn run(&self) -> Result<(), app::Error> {
        if self.native {
            if menu::refs().is_enabled() {
                return Ok(());
            }

            if !clipboard::is_cmd_installed() {
                crate::sys_log!(
                    "w",
                    "File yank failed: native command not installed, and config the native-clip is enabled"
                );
                crate::log!("Yank failed: command not installed (ex: wl-clip, xclip)");

                return Ok(());
            }

            let cursor = cursor::load();

            if cursor::is_selection() {
                let selected_files = misc::sorted_child_files(&app::get_path())
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, f)| cursor::is_selected(i).then_some(f))
                    .map(|f| format!("file://{}", f.to_string_lossy()))
                    .collect::<Vec<_>>();

                if let Err(e) = clipboard::clip_native(&selected_files.join("\n"), "text/uri-list")
                {
                    crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                    crate::log!("Yank failed: {}", e.kind());

                    return Ok(());
                }

                cursor::disable_selection();
                crate::sys_log!("i", "{} files yanked", selected_files.len());
                crate::log!("Yanked {} items", selected_files.len());

                return Ok(());
            }

            if let Some(under_cursor_file) =
                misc::sorted_child_files(&app::get_path()).get(cursor.current())
            {
                let text = format!("file://{}", under_cursor_file.to_string_lossy());

                if let Err(e) = clipboard::clip_native(&text, "text/uri-list") {
                    crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                    crate::log!("Yank failed: {}", e.kind());

                    return Ok(());
                }

                crate::sys_log!(
                    "i",
                    "File the {} yanked",
                    under_cursor_file.to_string_lossy()
                );
                crate::log!("Yanked \"{}\"", misc::file_name(under_cursor_file));
            }

            Ok(())
        } else {
            if menu::refs().is_enabled() {
                return Ok(());
            }

            let cursor = cursor::load();

            if cursor::is_selection() {
                let selected_files = misc::sorted_child_files(&app::get_path())
                    .into_iter()
                    .enumerate()
                    .filter_map(|(i, f)| {
                        cursor::is_selected(i).then_some(f.to_string_lossy().to_string())
                    })
                    .collect::<Vec<_>>();

                clipboard::clip(&selected_files.join("\n"));

                cursor::disable_selection();
                crate::sys_log!("i", "{} files yanked", selected_files.len());
                crate::log!("Yanked {} items", selected_files.len());

                return Ok(());
            }

            if let Some(under_cursor_file) =
                misc::sorted_child_files(&app::get_path()).get(cursor.current())
            {
                clipboard::clip(&under_cursor_file.to_string_lossy());

                crate::sys_log!(
                    "i",
                    "File the {} yanked",
                    under_cursor_file.to_string_lossy()
                );
                crate::log!("Yanked \"{}\"", misc::file_name(under_cursor_file));
            }

            Ok(())
        }
    }
}

pub struct Paste;

impl Command for Paste {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        if !clipboard::is_cmd_installed() {
            crate::sys_log!(
                "w",
                "File paste failed: native command not installed, and config the native-clip is enabled"
            );
            crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");

            return Ok(());
        }

        input::use_f_mut(|i| {
            let config = config::load();
            let default_paste_input = if config.paste.default_overwrite {
                "y"
            } else {
                ""
            };

            i.enable(default_paste_input, Some("Paste".into()));

            if config.paste.force_mode {
                crate::handler::handle_input_mode(
                    i,
                    crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
                );
            } else {
                crate::log!("overwrite the same files? (y/Y)");
            };
        });

        Ok(())
    }
}

pub struct Search {
    pub new: bool,
}

impl Command for Search {
    fn run(&self) -> Result<(), app::Error> {
        if menu::refs().is_enabled() {
            return Ok(());
        }

        cursor::disable_selection();

        if self.new {
            app::grep_update(|m| m.clear());
            input::use_f_mut(|i| i.enable("/", Some("Search".to_string())));
        } else if !app::is_regex_empty() {
            input::use_f_mut(|i| {
                i.enable("/", Some("Search".to_string()));
                crate::handler::handle_input_mode(
                    i,
                    crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Enter),
                )
            });
        }

        Ok(())
    }
}
