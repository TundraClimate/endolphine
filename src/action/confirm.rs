use crate::{
    action::Action,
    shell,
    ui::{self, Dialog},
    App,
};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
use std::{io, path::PathBuf};

pub fn pre_confirm(app: &mut App) -> io::Result<Action> {
    if let Some(dialog) = &app.dialog {
        let (_, rows) = terminal::size()?;
        execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine))?;
        if dialog.input.value().is_empty() {
            app.dialog = None;
            Ok(Action::None)
        } else {
            Ok(Action::Confirm)
        }
    } else {
        Ok(Action::PreConfirm)
    }
}

pub fn confirm(app: &mut App) -> io::Result<Action> {
    if let Some(Dialog { action, input }) = &app.dialog {
        let value = input.value();
        match action {
            Action::Create => confirm_create(value, &app.path.join(value))?,
            Action::Delete => {
                confirm_delete(app, value)?;
                app.selected.clear();
                app.cursor = 0;
            }
            Action::Rename => {
                if let Some(file) = app.cur_file() {
                    confirm_rename(value, file, &app.path.join(value))?
                }
            }
            Action::Search => {
                confirm_search(app.files.len())?;
                app.cursor = 0;
            }
            _ => {}
        }
    }
    app.dialog = None;
    Ok(Action::None)
}

fn confirm_create(value: &str, path: &PathBuf) -> io::Result<()> {
    if let Some(suff) = value.chars().last() {
        let operate = if suff == '/' {
            shell::mkdir
        } else {
            shell::create_file
        };
        operate(path);
        ui::log(format!("\"{}\" created", value))?;
    }
    Ok(())
}

fn confirm_delete(app: &App, value: &str) -> io::Result<()> {
    if value == "y" || value == "Y" {
        if app.selected.is_empty() {
            if let Some(file) = app.files.require(app.cursor) {
                ui::log(format!("\"{}\" deleted", crate::filename(&file)))?;
                shell::trash_file(&file);
            }
        } else {
            ui::log(format!("{} items deleted", app.selected.len()))?;
            app.selected.iter().for_each(|i| {
                if let Some(file) = app.files.require(*i) {
                    shell::trash_file(file);
                }
            });
        }
    }

    Ok(())
}

fn confirm_rename(value: &str, cur_file: &PathBuf, renamed: &PathBuf) -> io::Result<()> {
    if crate::filename(&cur_file) != value {
        ui::log(format!(
            "{} renamed \"{}\"",
            crate::filename(&cur_file),
            value
        ))?;
        shell::move_file(&cur_file, renamed);
    }
    Ok(())
}

fn confirm_search(files_len: usize) -> io::Result<()> {
    ui::log(format!("{} results found", files_len))?;
    Ok(())
}
