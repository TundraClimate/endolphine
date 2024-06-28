use crate::{
    action::Action,
    ui::{self, Dialog},
    App,
};
use std::io;

pub fn create(app: &mut App) -> io::Result<Action> {
    ui::log("".into())?;
    let dialog = Dialog::from(Action::Create);
    dialog.write_backend("New file/directory:")?;
    app.dialog = Some(dialog);
    Ok(Action::Pending)
}

pub fn delete(app: &mut App) -> io::Result<Action> {
    ui::log("".into())?;
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

pub fn rename(app: &mut App) -> io::Result<Action> {
    ui::log("".into())?;
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
