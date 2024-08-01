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
pub mod visual;

use visual::VisualType;

pub enum Action {
    Previous(usize),
    Next(usize),
    Back,
    Open,
    Visual(VisualType),
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
    if app.finder.is_search() || !app.dialog.is_some() {
        app.finder.cancel_search();
    }
    app.dialog = None;
    app.selected.clear();
    app.menu = None;
    Ok(Action::None)
}

pub fn search(app: &mut App) -> io::Result<Action> {
    ui::log("".into())?;
    let dialog = Dialog::from(Action::Search);
    dialog.write_backend("/")?;
    app.dialog = Some(dialog);
    app.finder.search("");
    Ok(Action::Pending)
}
