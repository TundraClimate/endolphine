use crate::{proc::CommandContext, state::State};
use std::sync::Arc;

pub fn toggle_menu_open(state: Arc<State>) {
    use crate::state::Mode;

    if state.flag.is_sidemenu_opened.get() {
        state.flag.is_sidemenu_opened.down();
        state.mode.switch(Mode::Normal);
        log::info!("The sidemenu is closed");
    } else {
        state.flag.is_sidemenu_opened.up();
        state.mode.switch(Mode::Menu);
        log::info!("The sidemenu is opened");
    }
}

pub fn toggle_menu(state: Arc<State>) {
    use crate::state::Mode;

    if state.flag.is_sidemenu_opened.get() {
        if state.mode.get() == Mode::Normal {
            state.mode.switch(Mode::Menu);
            log::info!("The sidemenu is enabled");
        } else {
            state.mode.switch(Mode::Normal);
            log::info!("The sidemenu is disabled");
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
        log::info!("Menu cursor move down by {point}");
    } else {
        cursor.shift_n(point);
        log::info!("Menu cursor move up by {point}");
    }
}

pub fn move_cursor_too(state: Arc<State>, positive: bool) {
    let cursor = &state.sidemenu.cursor;
    let point = cursor.len();

    if positive {
        cursor.shift_p(point);
        log::info!("Menu cursor move to bottom");
    } else {
        cursor.shift_n(point);
        log::info!("Menu cursor move to top");
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

    log::info!("Entering {} from Menu", element.tag);

    let path = &element.path;

    if !path.exists() {
        crate::log!("'{}' is not exists", element.tag);
        log::warn!("The sidemenu item '{}' is not exists", element.tag);
        log::warn!("{}:{}", element.tag, element.path.to_string_lossy());

        return;
    }

    if !path.is_dir() {
        crate::log!("'{}' is not Directory", element.tag);
        log::warn!("The sidemenu item '{}' is not Directory", element.tag);
        log::warn!("{}:{}", element.tag, element.path.to_string_lossy());

        return;
    }

    view::move_dir(state, path);
}
