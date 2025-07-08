use crate::{proc::CommandContext, state::State};
use std::sync::Arc;

pub fn move_cursor(state: Arc<State>, ctx: CommandContext, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = ctx.prenum.unwrap_or(1);

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}

pub fn move_cursor_too(state: Arc<State>, positive: bool) {
    let cursor = &state.file_view.cursor;
    let point = cursor.len();

    if positive {
        cursor.shift_p(point);
    } else {
        cursor.shift_n(point);
    }
}
