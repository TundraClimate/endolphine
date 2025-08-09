use super::Rect;
use crate::canvas;

pub(super) struct LogArea;

impl LogArea {
    pub(super) const ID: u8 = 5;

    pub(super) fn new() -> Self {
        Self
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crossterm::style::ResetColor;

        canvas::printin(
            rect,
            (0, 0),
            format!("{} {}", ResetColor, " ".repeat(rect.width.into())),
        )
    }
}
