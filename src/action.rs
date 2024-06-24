use crate::{
    app::App,
    shell,
    ui::{self, Dialog},
};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
use image::io::Reader as ImageReader;
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

pub enum Action {
    Previous(usize),
    Next(usize),
    Back,
    Open,
    Create,
    Delete,
    Cut,
    Copy,
    Paste,
    Rename,
    Pending,
    PreConfirm,
    Confirm,
    Clean,
    None,
}

pub fn previous(app: &mut App, i: usize) -> Action {
    let cursor = app.cursor;
    app.cursor = if cursor >= i { cursor - i } else { 0 };
    Action::None
}

pub fn next(app: &mut App, i: usize) -> Action {
    let cursor = app.cursor;
    let len = app.files.len();
    app.cursor = if cursor + i < len {
        cursor + i
    } else {
        len - 1
    };
    Action::None
}

pub fn back(app: &mut App) -> Action {
    if let Some(parent) = app.path.parent() {
        app.path = parent.to_path_buf();
        app.cursor = 0;
        app.selected.clear();
    }
    Action::None
}

pub fn open(app: &mut App) -> io::Result<Action> {
    let cur_position = app.cur_file();
    if cur_position.exists() {
        if cur_position.is_dir() {
            app.path = cur_position.clone();
            app.cursor = 0;
            app.selected.clear();
        } else {
            let mut file = File::open(cur_position)?;
            let mut buffer = [0; 1024];
            let read = file.read(&mut buffer)?;
            if std::str::from_utf8(&buffer[..read]).is_ok() {
                app.editor = true;
            } else if ImageReader::open(cur_position)?
                .with_guessed_format()?
                .format()
                .is_some()
            {
                shell::eog(cur_position)?;
            }
        }
    } else {
        ui::log(format!(
            "\"{}\" is not exists",
            crate::filename(&cur_position),
        ))?;
    }
    Ok(Action::None)
}

pub fn create(app: &mut App) -> io::Result<Action> {
    let dialog = Dialog::from(Action::Create);
    dialog.write_backend("New file/directory:")?;
    app.dialog = Some(dialog);
    Ok(Action::Pending)
}

pub fn delete(app: &mut App) -> io::Result<Action> {
    let dialog = Dialog::from(Action::Delete);
    if app.selected.is_empty() {
        dialog.write_backend(format!(
            "Delete \"{}\" ? (y/N)",
            crate::filename(app.cur_file())
        ))?;
    } else {
        let len = app.selected.len();
        dialog.write_backend(format!("Delete {} items? (y/N)", len))?;
    }
    app.dialog = Some(dialog);
    Ok(Action::Pending)
}

pub fn cut(app: &mut App) -> Action {
    app.is_cut = true;
    Action::Copy
}

pub fn copy(app: &mut App) -> io::Result<Action> {
    app.register.clear();
    if app.selected.is_empty() {
        let file = app.cur_file().clone();
        ui::log(format!("\"{}\" copied", crate::filename(&file)))?;
        app.register.push(file);
    } else {
        ui::log(format!("{} items copied", app.selected.len()))?;
        app.selected
            .iter()
            .for_each(|i| app.register.push(app.files[*i].clone()));
        app.selected.clear();
    }
    shell::clip(&app.register)?;
    Ok(Action::None)
}

pub fn paste(app: &mut App) -> io::Result<Action> {
    let register = &mut app.register;
    let current_dir = &app.path;
    let operate = if app.is_cut { shell::mv } else { shell::cp };
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

pub fn rename(app: &mut App) -> io::Result<Action> {
    let name = crate::filename(app.cur_file());
    let dialog = Dialog {
        action: Action::Rename,
        input: name.into(),
    };
    dialog.write_backend(format!("Rename \"{}\" :", name))?;
    app.dialog = Some(dialog);
    Ok(Action::Pending)
}

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
        let file = app.cur_file();
        match action {
            Action::Create => confirm_create(value, &app.path.join(value))?,
            Action::Delete => {
                confirm_delete(value, file, &app.files, &app.selected)?;
                app.selected.clear();
            }
            Action::Rename => confirm_rename(value, file, &app.path.join(value))?,
            _ => {}
        }
    }
    app.dialog = None;
    Ok(Action::None)
}

pub fn clean(app: &mut App) -> io::Result<Action> {
    let (_, rows) = terminal::size()?;
    execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine))?;
    app.dialog = None;
    app.selected.clear();
    Ok(Action::None)
}

fn confirm_create(value: &str, path: &PathBuf) -> io::Result<()> {
    if let Some(suff) = value.chars().last() {
        let operate = if suff == '/' {
            shell::mkdir
        } else {
            shell::create
        };
        operate(path);
        ui::log(format!("\"{}\" created", value))?;
    }
    Ok(())
}

fn confirm_delete(
    value: &str,
    cur_file: &PathBuf,
    files: &Vec<PathBuf>,
    selected: &Vec<usize>,
) -> io::Result<()> {
    if value == "y" || value == "Y" {
        if selected.is_empty() {
            let file = cur_file;
            ui::log(format!("\"{}\" deleted", crate::filename(&file)))?;
            shell::trash_file(&file);
        } else {
            ui::log(format!("{} items deleted", selected.len()))?;
            selected.iter().for_each(|i| shell::trash_file(&files[*i]));
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
        shell::mv(&cur_file, renamed);
    }
    Ok(())
}
