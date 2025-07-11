use super::Rect;
use crate::canvas;
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
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.wd.to_string_lossy().to_string().hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crate::{config, misc};
        use crossterm::style::{SetBackgroundColor, SetForegroundColor};

        let wd = &self.wd;
        let entry_name = format!("{}/", misc::entry_name(wd));
        let theme = &config::get().theme;
        let display_path = if let Some(parent) = wd.parent() {
            let usr = option_env!("USER").map_or("/root".to_string(), |u| match u {
                "root" => "/root".to_string(),
                user => format!("/home/{user}"),
            });
            let replaced = parent.to_string_lossy().replacen(&usr, "~", 1);

            format!(
                "{}{}{}{}{}",
                SetForegroundColor(theme.pwd_view.into()),
                replaced,
                if replaced.as_str() == "/" { "" } else { "/" },
                SetForegroundColor(theme.pwd_pickouted.into()),
                entry_name
            )
        } else {
            format!("{}/", SetForegroundColor(theme.pwd_pickouted.into()))
        };

        canvas::print_in(
            rect,
            0,
            0,
            &format!(
                "{}{} {} in {}{}",
                SetForegroundColor(theme.pwd_view.into()),
                SetBackgroundColor(theme.app_bg.into()),
                entry_name,
                display_path,
                " ".repeat(rect.width.into())
            ),
        );
    }
}
