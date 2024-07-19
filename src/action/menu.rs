use crate::{action::Action, command, file_manager, ui, App};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

pub fn open(app: &mut App) -> Action {
    if let Some(cur_file) = app.cur_file() {
        if !app.menu_opened() {
            app.menu = Some(cur_file.clone());
            app.cursor = 0;
        }
        Action::Pending
    } else {
        Action::None
    }
}

pub fn select(app: &mut App) -> io::Result<Action> {
    if let Some(selected) = app.cur_file() {
        handle_menu(selected, &app.menu)?;
    }
    app.cursor = 0;
    Ok(Action::Clean)
}

fn handle_menu(selected: &PathBuf, menu: &Option<PathBuf>) -> io::Result<()> {
    let name = crate::filename(selected);
    if let Some(ref path) = menu {
        handle_choice(name, path)?;
    }
    Ok(())
}

fn handle_choice(name: &str, path: &PathBuf) -> io::Result<()> {
    match name {
        "Create archive(.zip)" => {
            ui::log(format!("Archive \"{}\"...", crate::filename(path)))?;
            file_manager::zip(path.clone());
        }
        "Create archive(.tar.gz)" => {
            ui::log(format!("Archive \"{}\"...", crate::filename(path)))?;
            file_manager::tgz(path.clone());
        }
        "Extract from archive(Only .zip, .tar.gz)" => {
            file_manager::extract_from_archive(path.clone());
            ui::log(format!(
                "Extracting archive \"{}\"...",
                crate::filename(path)
            ))?;
        }
        "Open gimp" => {
            command::gimp(path)?;
            ui::log(format!("\"{}\" opened gimp", crate::filename(path)))?;
        }
        _ => {}
    }
    Ok(())
}

pub fn choices(path: &PathBuf) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = vec!["Create archive(.zip)", "Create archive(.tar.gz)"]
        .into_iter()
        .map(|s| PathBuf::from(s))
        .collect();
    if path.is_dir() {
        return Ok(files);
    }

    let mut file = File::open(path)?;
    let mut buffer = [0; 1024];
    let _ = file.read(&mut buffer)?;

    if file_manager::is_compressed(path)? {
        vec!["Extract from archive(Only .zip, .tar.gz)"]
            .into_iter()
            .map(|s| PathBuf::from(s))
            .for_each(|p| files.push(p));
    };
    if file_manager::is_image(path)? {
        vec!["Open gimp"]
            .into_iter()
            .map(|s| PathBuf::from(s))
            .for_each(|p| files.push(p));
    }
    Ok(files)
}
