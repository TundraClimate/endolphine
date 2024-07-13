use crate::{action::Action, shell, ui, App};
use std::io;

pub fn cut(app: &mut App) -> Action {
    app.is_cut = true;
    Action::Copy
}

pub fn copy(app: &mut App) -> io::Result<Action> {
    app.register.clear();
    if app.selected.is_empty() {
        if let Some(file) = app.cur_file() {
            ui::log(format!("\"{}\" copied", crate::filename(&file)))?;
            app.register.push(file.clone());
        }
    } else {
        ui::log(format!("{} items copied", app.selected.len()))?;
        app.selected.iter().for_each(|i| {
            if let Some(file) = app.files.require(*i) {
                app.register.push(file.clone());
            }
        });
        app.selected.clear();
    }
    shell::clip(&app.register)?;
    Ok(Action::None)
}

pub fn paste(app: &mut App) -> io::Result<Action> {
    let register = &mut app.register;
    let current_dir = &app.path;
    let operate = if app.is_cut {
        shell::move_file
    } else {
        shell::copy_file
    };
    register.iter().for_each(|p| {
        if let Some(parent) = p.parent() {
            if parent != current_dir {
                operate(p, current_dir);
            } else {
                let mut modif = current_dir.clone();
                modif.push(format!("{}(Copy)", crate::filename(&p)));
                operate(p, &modif);
            }
        }
    });

    ui::log(format!("{} items pasted", register.len()))?;

    if app.is_cut {
        register.clear();
        app.is_cut = false;
    }
    Ok(Action::None)
}
