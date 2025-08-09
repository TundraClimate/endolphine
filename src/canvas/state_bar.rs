use super::Rect;
use crate::{canvas, state::Mode};

pub(super) struct StateBar {
    mode: Mode,
    procs: usize,
}

impl StateBar {
    pub(super) const ID: u8 = 4;

    pub(super) fn new(mode: Mode, procs: usize) -> Self {
        Self { mode, procs }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.mode.hash(&mut hasher);
        self.procs.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crate::config;
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let theme = &config::get().theme;

        let current_mode = match self.mode {
            Mode::Normal => format!("{} NORMAL ", SetBackgroundColor(theme.mode_normal.into())),
            Mode::Visual => format!("{} VISUAL ", SetBackgroundColor(theme.mode_visual.into())),
            Mode::Input => format!("{} INPUT ", SetBackgroundColor(theme.mode_input.into())),
            Mode::Search => format!("{} SEARCH ", SetBackgroundColor(theme.mode_search.into())),
            Mode::Menu => format!("{} MENU ", SetBackgroundColor(theme.mode_menu.into())),
        };

        canvas::printin(
            rect,
            (0, 0),
            format!(
                "{}{}{} {} procs running{}",
                SetForegroundColor(theme.bar_fg.into()),
                current_mode,
                SetBackgroundColor(theme.bar_bg.into()),
                self.procs,
                " ".repeat(rect.width.into())
            ),
        );
    }
}
