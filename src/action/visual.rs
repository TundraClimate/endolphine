use crate::{action::Action, App};

#[derive(Clone, Copy)]
pub enum VisualType {
    Cursor,
    All,
}

pub fn visual_select(app: &mut App, v: VisualType) -> Action {
    match v {
        VisualType::Cursor => {
            app.selected.push(app.cursor);
            Action::None
        }
        VisualType::All => {
            if app.finder.len() != 0 {
                app.cursor = 0;
                app.selected.push(app.cursor);
                Action::Next(app.finder.len() - 1)
            } else {
                Action::None
            }
        }
    }
}
