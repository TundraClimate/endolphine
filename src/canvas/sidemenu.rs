use super::Rect;
use crate::canvas;

pub(super) struct Sidemenu {}

impl Sidemenu {
    pub(super) const ID: u8 = 0;

    pub(super) fn new() -> Self {
        Self {}
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        // mock
        for i in 0..rect.height {
            canvas::print_in(rect, 0, i, &" ".repeat(rect.width.into()));
        }
    }
}
