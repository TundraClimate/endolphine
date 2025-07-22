use super::Rect;
use crate::canvas;

pub(super) struct LogArea {
    input_buf: String,
    input_tag: Option<String>,
}

impl LogArea {
    pub(super) const ID: u8 = 5;

    pub(super) fn new(input_buf: String, input_tag: Option<String>) -> Self {
        Self {
            input_buf,
            input_tag,
        }
    }

    pub(super) fn make_hash(&self, layout_hash: u64) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        layout_hash.hash(&mut hasher);
        self.input_buf.hash(&mut hasher);
        self.input_tag.hash(&mut hasher);

        hasher.finish()
    }

    pub(super) fn draw(&self, rect: Rect) {
        use crossterm::style::ResetColor;

        let input_buf = if let Some(ref tag) = self.input_tag {
            let (tag, ctx) = tag.split_once(":").unwrap_or((tag, ""));

            let prefix = match tag {
                "DeleteThisItem" => &format!("Delete the '{ctx}' (y/N)"),
                "DeleteItems" => {
                    let Some((count, _)) = ctx.split_once(";") else {
                        panic!("Cannot parse the 'DeleteItems' context");
                    };

                    &format!("Delete {count} items (y/N)")
                }
                "PasteFromCb" => "Overwrite a file (Y/n)",

                _ => return,
            };

            &format!("{}: {}", prefix, self.input_buf)
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
