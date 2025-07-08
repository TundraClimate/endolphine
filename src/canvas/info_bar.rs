use super::Rect;
use crate::canvas;
use std::path::PathBuf;

pub(super) struct InfoBar {
    wd: PathBuf,
    cursor_pos: usize,
    file_view_len: usize,
}

impl InfoBar {
    pub(super) const ID: u8 = 2;

    pub(super) fn new(wd: PathBuf, cursor_pos: usize, file_view_len: usize) -> Self {
        Self {
            wd,
            cursor_pos,
            file_view_len,
        }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.wd.to_string_lossy().to_string().hash(&mut hasher);
        self.cursor_pos.hash(&mut hasher);
        self.file_view_len.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crate::{config, misc};
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let theme = &config::get().theme;
        let page = self.cursor_pos / self.file_view_len.max(1) + 1;
        let files_len = misc::child_files_len(&self.wd);

        canvas::print_in(
            rect,
            0,
            0,
            &format!(
                "{}{} Page {} {}(All {} items){}",
                SetBackgroundColor(theme.bar_bg.into()),
                SetForegroundColor(theme.bar_fg.into()),
                page,
                SetForegroundColor(theme.bar_fg_light.into()),
                files_len,
                " ".repeat(rect.width.into())
            ),
        );
    }
}
