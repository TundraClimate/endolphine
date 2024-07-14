use crate::{action::Action, shell, ui, App};
use std::{io, path::PathBuf};

pub fn cut(app: &mut App) -> Action {
    app.is_cut = true;
    Action::Copy
}

pub fn copy(app: &mut App) -> io::Result<Action> {
    if app.selected.is_empty() {
        if let Some(file) = app.cur_file() {
            ui::log(format!("\"{}\" copied", crate::filename(&file)))?;
            shell::clip(vec![file])?;
        }
    } else {
        ui::log(format!("{} items copied", app.selected.len()))?;
        let files: Vec<_> = app
            .selected
            .iter()
            .filter_map(|i| app.files.require(*i))
            .collect();
        shell::clip(files)?;
        app.selected.clear();
    }
    Ok(Action::None)
}

pub fn paste(app: &mut App) -> io::Result<Action> {
    let clipboard = shell::clipboard()?;
    if clipboard.starts_with("file://") {
        let clipboard: Vec<_> = clipboard.split("\n").collect();
        let pathes: Vec<_> = clipboard
            .iter()
            .filter_map(|s| s.strip_prefix("file://"))
            .map(|s| PathBuf::from(s))
            .filter(|p| p.exists())
            .collect();
        if pathes.len() != clipboard.len() {
            return Ok(Action::None);
        }
        let operate = if app.is_cut {
            shell::move_file
        } else {
            shell::copy_file
        };
        pathes.iter().for_each(|p| {
            if let Some(parent) = p.parent() {
                if parent != app.path {
                    operate(p, &app.path);
                } else {
                    if !app.is_cut {
                        let mut modif = app.path.clone();
                        modif.push(format!("{}(Copy)", crate::filename(&p)));
                        operate(p, &modif);
                    }
                }
            }
        });
        ui::log(format!("pasted {} items", clipboard.len()))?;
        if app.is_cut {
            app.is_cut = false;
            shell::clean_clipboard()?;
        }
    }
    Ok(Action::None)
}
