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
        cursor.shift(pos as isize);
        cache.unwrap_surface();
    } else {
        cache.reset();
    }
}

fn enter_file(target_path: &std::path::Path) -> Result<(), app::Error> {
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
        .map_err(|e| app::Error::CommandExecutionFailed(cmd.to_owned(), e.kind().to_string()))?;

    if in_term {
        app::enable_tui()?;
        canvas::cache_clear();
    }

    Ok(())
}
