use crate::{proc::CommandContext, state::State};
use std::sync::Arc;

pub fn toggle_menu_open(state: Arc<State>) {
    use crate::state::Mode;

    if state.flag.is_sidemenu_opened.get() {
        state.flag.is_sidemenu_opened.down();
        state.mode.switch(Mode::Normal);
    } else {
        state.flag.is_sidemenu_opened.up();
        state.mode.switch(Mode::Menu);
    }
}

pub fn toggle_menu(state: Arc<State>) {
    use crate::state::Mode;

    if state.flag.is_sidemenu_opened.get() {
        if state.mode.get() == Mode::Normal {
            state.mode.switch(Mode::Menu);
        } else {
            state.mode.switch(Mode::Normal);
        }
    } else {
        toggle_menu_open(state);
    }
}

pub fn move_cursor(state: Arc<State>, ctx: CommandContext, positive: bool) {
    let cursor = &state.sidemenu.cursor;
    let point = ctx.prenum.unwrap_or(1);

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

pub fn move_cursor_too(state: Arc<State>, positive: bool) {
    let cursor = &state.sidemenu.cursor;
    let point = cursor.len();

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

pub fn enter(state: Arc<State>) {
    use super::view;
    use crate::config;

    let cursor = &state.sidemenu.cursor;
    let config = config::get();

    let Some(element) = config.menu_elements.get(cursor.current()) else {
        return;
    };

    let path = &element.path;

    if !path.exists() {
        crate::log!("'{}' is not exists", element.tag);

        return;
    }

    if !path.is_dir() {
        crate::log!("'{}' is not Directory", element.tag);

        return;
    }

    view::move_dir(state, path);
}
