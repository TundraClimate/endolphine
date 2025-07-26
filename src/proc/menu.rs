use crate::state::State;
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
