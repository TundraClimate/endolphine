use super::Command;
use crate::{app, canvas, config, cursor, menu, misc};

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

pub struct MoveDown;

impl Command for MoveDown {
    fn run(&self) -> Result<(), app::Error> {
        let cursor = cursor::captured();

        let prenum = app::load_buf()
            .into_iter()
            .take_while(crate::key::Key::is_digit)
            .map(|k| k.as_num())
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, k)| {
                if i > 3 {
                    0
                } else {
                    (k - 48) * (10u8.pow(i as u32))
                }
            })
            .sum::<u8>();
        let mv_len = if prenum == 0 { 1 } else { prenum.into() };

        cursor.shift_p(mv_len);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor.current());
        }

        Ok(())
    }
}

pub struct MoveUp;

impl Command for MoveUp {
    fn run(&self) -> Result<(), app::Error> {
        let cursor = cursor::captured();

        let prenum = app::load_buf()
            .into_iter()
            .take_while(crate::key::Key::is_digit)
            .map(|k| k.as_num())
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, k)| {
                if i > 3 {
                    0
                } else {
                    (k - 48) * (10u8.pow(i as u32))
                }
            })
            .sum::<u8>();
        let mv_len = if prenum == 0 { 1 } else { prenum.into() };

        cursor.shift_n(mv_len);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor.current());
        }

        Ok(())
    }
}

pub struct MoveTop;

impl Command for MoveTop {
    fn run(&self) -> Result<(), crate::app::Error> {
        cursor::captured().reset();

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor::load().current());
        }

        Ok(())
    }
}

pub struct MoveBottom;

impl Command for MoveBottom {
    fn run(&self) -> Result<(), crate::app::Error> {
        let len = misc::child_files_len(&app::get_path());
        cursor::captured().shift_p(len);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor::load().current());
        }

        Ok(())
    }
}

pub struct PageDown;

impl Command for PageDown {
    fn run(&self) -> Result<(), crate::app::Error> {
        let prenum = app::load_buf()
            .into_iter()
            .take_while(crate::key::Key::is_digit)
            .map(|k| k.as_num())
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, k)| {
                if i > 3 {
                    0
                } else {
                    (k - 48) * (10u8.pow(i as u32))
                }
            })
            .sum::<u8>();
        let page = if prenum == 0 { 1 } else { prenum };
        let page_len = misc::body_height() as usize;

        cursor::captured().shift_p(page as usize * page_len);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor::load().current());
        }

        Ok(())
    }
}

pub struct PageUp;

impl Command for PageUp {
    fn run(&self) -> Result<(), crate::app::Error> {
        let prenum = app::load_buf()
            .into_iter()
            .take_while(crate::key::Key::is_digit)
            .map(|k| k.as_num())
            .collect::<Vec<u8>>()
            .into_iter()
            .rev()
            .enumerate()
            .map(|(i, k)| {
                if i > 3 {
                    0
                } else {
                    (k - 48) * (10u8.pow(i as u32))
                }
            })
            .sum::<u8>();
        let page = if prenum == 0 { 1 } else { prenum.into() };
        let page_len = misc::body_height() as usize;

        cursor::captured().shift_n(page * page_len);

        if cursor::is_selection() && !menu::refs().is_enabled() {
            cursor::select_area(cursor::load().current());
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
            cursor.shift_p(pos);
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
            enter_menu_element(menu, cursor);

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
            enter_dir(target_path, cursor);
        } else if target_path.is_file() {
            enter_file(target_path)?;
        }

        Ok(())
    }
}

fn enter_menu_element(menu: &menu::Menu, cursor: &cursor::Cursor) {
    if let Some(element) = menu.elements.get(cursor.current()) {
        let path = &element.path;

        if !path.is_dir() {
            crate::sys_log!("w", "Found the invalid Shortcut in MENU: {}", element.tag);
            crate::log!("\"{}\" is not Directory", element.tag);

            return;
        }

        move_current_dir(path);
        menu.toggle_enable();
        cursor::load().cache.write().unwrap().reset();
    }
}

fn enter_dir(target_path: &std::path::Path, cursor: &cursor::Cursor) {
    let child_files = misc::sorted_child_files(target_path);

    move_current_dir(target_path);

    let mut cache = cursor.cache.write().unwrap();

    if let Some(pos) = child_files.iter().position(|e| cache.inner_equal(e)) {
        cursor.shift_p(pos);
        cache.unwrap_surface();
    } else {
        cache.reset();
    }
}

fn enter_file(target_path: &std::path::Path) -> Result<(), app::Error> {
    let mut cmd = config::load().editor.clone();
    let mut in_term = true;

    if let Some(extension) = target_path.extension().map(|e| e.to_string_lossy()) {
        if let Some(opts) = config::load()
            .open
            .as_ref()
            .and_then(|o| o.corresponding_with(&extension))
        {
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
        .map_err(|e| app::Error::CommandExecutionFailed(cmd.to_owned(), e.kind().to_string()))?;

    if in_term {
        app::enable_tui()?;
        canvas::cache_clear();
    }

    Ok(())
}
