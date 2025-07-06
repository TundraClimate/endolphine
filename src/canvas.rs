use crate::state::State;
use std::sync::Arc;

pub fn draw(state: Arc<State>) {}

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
            height: term_rect.height.min(2),
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
            height: term_rect.height.min(2),
        },
    ]);

    Layout::new(layout)
}
