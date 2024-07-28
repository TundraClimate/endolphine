use crate::{action::Action, command, ui, App};
use std::{io, path::PathBuf};

pub fn cut(app: &mut App) -> Action {
    app.is_cut = true;
    Action::Copy
}

pub fn copy(app: &mut App) -> io::Result<Action> {
    if app.selected.is_empty() {
        if let Some(file) = app.cur_file() {
            ui::log(format!("\"{}\" copied", crate::filename(&file)))?;
            command::clip(vec![file])?;
        }
    } else {
        ui::log(format!("{} items copied", app.selected.len()))?;
        let files: Vec<_> = app
            .selected
            .iter()
            .filter_map(|i| app.finder.require(*i))
            .collect();
        command::clip(files)?;
        app.selected.clear();
    }
    Ok(Action::None)
}

pub fn paste(app: &mut App) -> io::Result<Action> {
    let clipboard = command::clipboard()?;
    if !clipboard.starts_with("file://") {
        return Ok(Action::None);
    }
    let pathes: Vec<_> = clipboard
        .lines()
        .filter_map(|s| s.strip_prefix("file://"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();
    if pathes.len() != clipboard.lines().count() {
        return Ok(Action::None);
    }
    let operate = if app.is_cut {
        command::move_file
    } else {
        command::copy_file
    };
    pathes.iter().for_each(|p| {
        if let Some(parent) = p.parent() {
            if parent == app.path {
                if !app.is_cut {
                    let new_path = app.path.join(format!("{}(Copy)", crate::filename(&p)));
                    operate(p, &new_path);
                }
            } else {
                operate(p, &app.path);
            }
        }
    });
    ui::log(format!("pasted {} items", pathes.len()))?;
    if app.is_cut {
        app.is_cut = false;
        command::clean_clipboard()?;
    }
    Ok(Action::None)
}
