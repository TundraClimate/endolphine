use crate::{action::Action, file_manager, shell, ui, App};
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

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
    if let Some(cur_position) = app.cur_file() {
        let cur_position = cur_position.clone();
        if !cur_position.exists() {
            ui::log(format!(
                "\"{}\" is not exists",
                crate::filename(&cur_position),
            ))?;
            return Ok(Action::None);
        }
        if cur_position.is_dir() {
            open_dir(app, &cur_position);
            return Ok(Action::Clean);
        } else {
            open_file(app, &cur_position)?;
        }
    }
    Ok(Action::None)
}

fn open_file(app: &mut App, cur_file: &PathBuf) -> io::Result<()> {
    let mut file = File::open(cur_file)?;
    let mut buffer = [0; 1024];
    let read = file.read(&mut buffer)?;

    if std::str::from_utf8(&buffer[..read]).is_ok() {
        app.editor = true;
    } else if file_manager::is_image(cur_file)? {
        shell::eog(cur_file)?;
    } else if shell::ffprobe_is_video(cur_file) {
        shell::vlc(cur_file)?;
    } else if file_manager::is_compressed(cur_file)? {
        shell::file_roller_open(cur_file)?;
    } else if &buffer[..4] == b"%PDF" {
        shell::evince(cur_file)?;
    }
    Ok(())
}

fn open_dir(app: &mut App, cur_file: &PathBuf) {
    app.path = cur_file.clone();
    app.cursor = 0;
    app.selected.clear();
    app.is_search = false;
}
