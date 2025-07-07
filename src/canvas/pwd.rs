use super::Rect;
use std::path::PathBuf;

pub(super) struct Working {
    wd: PathBuf,
}

impl Working {
    pub(super) const ID: u8 = 1;

    pub(super) fn new(wd: PathBuf) -> Self {
        Self { wd }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        todo!()
    }

    pub(super) fn draw(&self, rect: Rect) {}
}
