mod info_bar;
mod pwd;

use crate::state::State;
use info_bar::InfoBar;
use pwd::Working;
use std::sync::Arc;

pub fn draw(state: Arc<State>) {
    let layout = gen_layout(state.term_size.load(), state.flag.is_sidemenu_opened.get());
    let layout_key = layout.hashcode();

    let hashes = &state.canvas_hashes;

    let working = Working::new(state.work_dir.get());

    if hashes.get(Working::ID) != Some(working.make_hash(layout_key)) {
        working.draw(layout.get(Working::ID));
    }

    let infobar = InfoBar::new(state.work_dir.get(), 0, layout.get(3).height.into());

    if hashes.get(InfoBar::ID) != Some(infobar.make_hash(layout_key)) {
        infobar.draw(layout.get(InfoBar::ID));
    }
}

fn print_in(rect: Rect, rel_x: u16, rel_y: u16, s: &str) {
    use crossterm::{
        cursor::MoveTo,
        style::{Print, ResetColor},
    };
    use std::io;
    use unicode_width::UnicodeWidthChar;

    if rect.height <= rel_y || rect.width <= rel_x {
        return;
    }

    let abs_x = rect.x + rel_x;
    let abs_y = rect.y + rel_y;
    let mut text = String::new();
    let mut rem = rect.width.saturating_sub(rel_x) as usize;
    let mut chars = s.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c == '\x1b' {
            let mut seq = String::new();

            while let Some(&next) = chars.peek() {
                seq.push(next);
                chars.next();
                if next == 'm' || next == 'K' {
                    break;
                }
            }

            text.push_str(&seq);
        } else {
            let w = UnicodeWidthChar::width(c).unwrap_or(0);

            if w > rem {
                text.push_str(&" ".repeat(rem));
                break;
            }

            rem -= w;
            text.push(c);
            chars.next();
        }
    }

    crossterm::queue!(io::stdout(), MoveTo(abs_x, abs_y), Print(text), ResetColor,).ok();
}

#[derive(Hash)]
pub struct Layout {
    areas: Vec<Rect>,
}

impl Layout {
    fn new(areas: Vec<Rect>) -> Self {
        Self { areas }
    }

    fn hashcode(&self) -> u64 {
        use std::hash::{DefaultHasher, Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }

    fn get(&self, index: u8) -> Rect {
        self.areas
            .get(index as usize)
            .copied()
            .expect("Invalid layout id detected")
    }
}

#[derive(Clone, Copy, Hash)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

fn gen_layout(term_rect: Rect, is_sidemenu_opened: bool) -> Layout {
    let mut term_rect = term_rect;
    let mut layout = vec![];

    if is_sidemenu_opened {
        term_rect = Rect {
            x: term_rect.x.saturating_add(20),
            y: term_rect.y,
            width: term_rect.width.saturating_sub(20),
            height: term_rect.height,
        };

        layout.push(Rect {
            x: term_rect.x,
            y: term_rect.y,
            width: 20,
            height: term_rect.height,
        });
    } else {
        layout.push(Rect {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });
    }

    layout.append(&mut vec![
        Rect {
            x: term_rect.x,
            y: term_rect.y,
            width: term_rect.width,
            height: term_rect.height.min(1),
        },
        Rect {
            x: term_rect.x,
            y: term_rect.y.saturating_add(1),
            width: term_rect.width,
            height: term_rect.height.min(1),
        },
        Rect {
            x: term_rect.x,
            y: term_rect.y.saturating_add(2),
            width: term_rect.width,
            height: term_rect.height.saturating_sub(4),
        },
        Rect {
            x: term_rect.x,
            y: term_rect.height.saturating_sub(2),
            width: term_rect.width,
            height: term_rect.height.min(1),
        },
        Rect {
            x: term_rect.x,
            y: term_rect.height.saturating_sub(1),
            width: term_rect.width,
            height: term_rect.height.min(1),
        },
    ]);

    Layout::new(layout)
}
