use super::Rect;
use crate::canvas;

pub(super) struct LogArea {
    is_input_enable: bool,
    input_buf: String,
}

impl LogArea {
    pub(super) const ID: u8 = 5;

    pub(super) fn new(is_input_enable: bool, input_buf: String) -> Self {
        Self {
            is_input_enable,
            input_buf,
        }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.is_input_enable.hash(&mut hasher);
        self.input_buf.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crossterm::style::ResetColor;

        let input_buf = if self.is_input_enable {
            &self.input_buf
        } else {
            ""
        };

        canvas::print_in(
            rect,
            0,
            0,
            &format!(
                "{} {}{}",
                ResetColor,
                input_buf,
                " ".repeat(rect.width.into())
            ),
        )
    }
}
