use crate::{
    app::App,
    file_manager::FileManager,
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
    Search,
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
    if len != 0 {
        app.cursor = if cursor + i < len {
            cursor + i
        } else {
            len - 1
        };
    }
    Action::None
}

pub fn back(app: &mut App) -> Action {
    if let Some(parent) = app.path.parent() {
        let before = &app.path.clone();
        app.path = parent.to_path_buf();
        let pathes = crate::dir_pathes(None, &app.path);
        let cursor = pathes
            .iter()
            .enumerate()
            .find_map(|(i, p)| if p == before { Some(i) } else { None });
        app.cursor = if let Some(cursor) = cursor { cursor } else { 0 };
    }
    Action::Clean
}

pub fn open(app: &mut App) -> io::Result<Action> {
    let cur_position = if let Some(p) = app.cur_file() {
        p.clone()
    } else {
        return Ok(Action::None);
    };
    if !cur_position.exists() {
        ui::log(format!(
            "\"{}\" is not exists",
            crate::filename(&cur_position),
        ))?;
        return Ok(Action::None);
    }
    if cur_position.is_dir() {
        open_dir(app, &cur_position);
    } else {
        open_file(app, &cur_position)?;
    }
    Ok(Action::None)
}

fn open_file(app: &mut App, cur_file: &PathBuf) -> io::Result<()> {
    let mut file = File::open(cur_file)?;
    let mut buffer = [0; 1024];
    let read = file.read(&mut buffer)?;

    let is_image = ImageReader::open(cur_file)?
        .with_guessed_format()?
        .format()
        .is_some();

    if std::str::from_utf8(&buffer[..read]).is_ok() {
        app.editor = true;
    } else if is_image {
        shell::eog(cur_file)?;
    } else if shell::ffprobe_is_video(cur_file) {
        shell::vlc(cur_file)?;
    }
    Ok(())
}

fn open_dir(app: &mut App, cur_file: &PathBuf) {
    app.path = cur_file.clone();
    app.cursor = 0;
    app.selected.clear();
    app.is_search = false;
}

pub fn create(app: &mut App) -> io::Result<Action> {
    let dialog = Dialog::from(Action::Create);
    dialog.write_backend("New file/directory:")?;
    app.dialog = Some(dialog);
    Ok(Action::Pending)
}

pub fn delete(app: &mut App) -> io::Result<Action> {
    let dialog = Dialog::from(Action::Delete);
    if let Some(file) = app.cur_file() {
        if app.selected.is_empty() {
            dialog.write_backend(format!("Delete \"{}\" ? (y/N)", crate::filename(file)))?;
        } else {
            let len = app.selected.len();
            dialog.write_backend(format!("Delete {} items? (y/N)", len))?;
        }
        app.dialog = Some(dialog);
        Ok(Action::Pending)
    } else {
        Ok(Action::None)
    }
}

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
            if let Some(file) = app.files.cur_file(*i) {
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
    if let Some(file) = app.cur_file() {
        let name = crate::filename(file);
        let dialog = Dialog {
            action: Action::Rename,
            input: name.into(),
        };
        dialog.write_backend(format!("Rename \"{}\" :", name))?;
        app.dialog = Some(dialog);
        Ok(Action::Pending)
    } else {
        Ok(Action::None)
    }
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
        match action {
            Action::Create => confirm_create(value, &app.path.join(value))?,
            Action::Delete => {
                confirm_delete(value, app.cursor, &app.files, &app.selected)?;
                app.selected.clear();
            }
            Action::Rename => {
                if let Some(file) = app.cur_file() {
                    confirm_rename(value, file, &app.path.join(value))?
                }
            }
            Action::Search => confirm_search(app.files.len())?,
            _ => {}
        }
    }
    if !app.is_search {
        app.dialog = None;
    }
    Ok(Action::None)
}

pub fn clean(app: &mut App) -> io::Result<Action> {
    let (_, rows) = terminal::size()?;
    execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine))?;
    app.dialog = None;
    app.selected.clear();
    app.is_search = false;
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
    cursor: usize,
    files: &FileManager,
    selected: &Vec<usize>,
) -> io::Result<()> {
    if value == "y" || value == "Y" {
        if selected.is_empty() {
            if let Some(file) = files.cur_file(cursor) {
                ui::log(format!("\"{}\" deleted", crate::filename(&file)))?;
                shell::trash_file(&file);
            }
        } else {
            ui::log(format!("{} items deleted", selected.len()))?;
            selected.iter().for_each(|i| {
                if let Some(file) = files.cur_file(*i) {
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
        shell::mv(&cur_file, renamed);
    }
    Ok(())
}

fn confirm_search(files_len: usize) -> io::Result<()> {
    ui::log(format!("{} results found", files_len))?;
    Ok(())
}

pub fn search(app: &mut App) -> io::Result<Action> {
    let dialog = Dialog::from(Action::Search);
    dialog.write_backend("/")?;
    app.dialog = Some(dialog);
    app.is_search = true;
    Ok(Action::Pending)
}
