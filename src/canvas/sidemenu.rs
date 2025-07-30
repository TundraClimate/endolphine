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
        use crate::config;
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let config = config::get();
        let theme = &config.theme;

        canvas::printin(
            rect,
            (0, 0),
            format!(
                "{}{} Select to Cd {}",
                SetBackgroundColor(theme.app_bg.into()),
                SetForegroundColor(theme.app_fg.into()),
                " ".repeat(rect.width.into()),
            ),
        );

        canvas::printin(
            rect,
            (0, 1),
            format!(
                "{}{}{}",
                SetBackgroundColor(theme.bar_bg.into()),
                SetForegroundColor(theme.bar_fg.into()),
                " ".repeat(rect.width.into()),
            ),
        );

        for i in 2..rect.height.saturating_sub(1) {
            let rel_i = i.saturating_sub(2) as usize;

            match config.menu_elements.get(rel_i) {
                Some(element) => {
                    canvas::printin(
                        rect,
                        (0, i),
                        format!(
                            "{}{} | {}{}{}",
                            SetBackgroundColor(theme.app_bg.into()),
                            SetForegroundColor(theme.app_fg.into()),
                            SetForegroundColor(theme.item_sidemenu.into()),
                            element.tag,
                            " ".repeat(rect.width.into()),
                        ),
                    );
                }
                None => {
                    canvas::printin(
                        rect,
                        (0, i),
                        format!(
                            "{}{}{}",
                            SetBackgroundColor(theme.app_bg.into()),
                            SetForegroundColor(theme.app_fg.into()),
                            " ".repeat(rect.width.into()),
                        ),
                    );
                }
            }
        }

        canvas::printin(
            rect,
            (0, rect.height.saturating_sub(1)),
            format!(
                "{}{}{}",
                SetBackgroundColor(theme.bar_bg.into()),
                SetForegroundColor(theme.bar_fg.into()),
                " ".repeat(rect.width.into()),
            ),
        );

        for i in 0..rect.height {
            canvas::printin(
                rect,
                (rect.width.saturating_sub(1), i),
                format!(
                    "{}{}|",
                    SetBackgroundColor(theme.bar_bg.into()),
                    SetForegroundColor(theme.bar_fg_light.into())
                ),
            );
        }
    }
}
