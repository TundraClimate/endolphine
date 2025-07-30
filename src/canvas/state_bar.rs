use super::Rect;
use crate::canvas;

pub(super) struct StateBar {
    procs: usize,
}

impl StateBar {
    pub(super) const ID: u8 = 4;

    pub(super) fn new(procs: usize) -> Self {
        Self { procs }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.procs.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crate::config;
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let theme = &config::get().theme;

        canvas::printin(
            rect,
            (0, 0),
            format!(
                "{}{} {} procs running{}",
                SetBackgroundColor(theme.bar_bg.into()),
                SetForegroundColor(theme.bar_fg.into()),
                self.procs,
                " ".repeat(rect.width.into())
            ),
        );
    }
}
