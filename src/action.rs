use crate::{
    app::App,
    ui::{self, Dialog},
};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io;

pub mod clip;
pub mod confirm;
pub mod manage;
pub mod menu;
pub mod move_h;
pub mod move_v;

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
    Menu,
    Select,
    Pending,
    PreConfirm,
    Confirm,
    Clean,
    None,
}

pub fn clean(app: &mut App) -> io::Result<Action> {
    let (_, rows) = terminal::size()?;
    execute!(io::stdout(), MoveTo(0, rows), Clear(ClearType::CurrentLine))?;
    app.dialog = None;
    app.selected.clear();
    app.is_search = false;
    app.menu = None;
    Ok(Action::None)
}

pub fn search(app: &mut App) -> io::Result<Action> {
    ui::log("".into())?;
    let dialog = Dialog::from(Action::Search);
    dialog.write_backend("/")?;
    app.dialog = Some(dialog);
    app.is_search = true;
    Ok(Action::Pending)
}
