mod info_bar;
mod log_area;
mod pwd;
mod state_bar;
mod viewer;

use self::{
    info_bar::InfoBar, log_area::LogArea, pwd::Working, state_bar::StateBar, viewer::Viewer,
};
use crate::state::State;
use std::sync::Arc;

pub fn draw(state: Arc<State>) {
    use std::io::{self, Write};

    let layout = gen_layout(state.term_size.load(), state.flag.is_sidemenu_opened.get());
    let layout_key = layout.hashcode();

    let hashes = &state.canvas_hashes;

    let working = Working::new(state.work_dir.get());
    let working_hash = working.make_hash(layout_key);

    if hashes.update(Working::ID, working_hash) != Some(working_hash) {
        working.draw(layout.get(Working::ID));
    }

    let infobar = InfoBar::new(
        state.work_dir.get(),
        state.file_view.cursor.current(),
        layout.get(Viewer::ID).height.into(),
    );
    let infobar_hash = infobar.make_hash(layout_key);

    if hashes.update(InfoBar::ID, infobar_hash) != Some(infobar_hash) {
        infobar.draw(layout.get(InfoBar::ID));
    }

    let viewer = Viewer::new(
        state.work_dir.get(),
        state.file_view.cursor.current(),
        state.file_view.selection.collect(),
        String::new(),
        state.input.tag(),
        state
            .input
            .is_enable()
            .then_some(state.input.input.buf_clone()),
        state.input.input.cursor(),
    );
    let viewer_hash = viewer.make_hash(layout_key);

    if hashes.update(Viewer::ID, viewer_hash) != Some(viewer_hash) {
        viewer.draw(layout.get(Viewer::ID));
    }

    let statebar = StateBar::new(state.proc_counter.now());
    let statebar_hash = statebar.make_hash(layout_key);

    if hashes.update(StateBar::ID, statebar_hash) != Some(statebar_hash) {
        statebar.draw(layout.get(StateBar::ID));
    }

    let logarea = LogArea::new(state.input.input.buf_clone(), state.input.tag());
    let logarea_hash = logarea.make_hash(layout_key);

    if hashes.update(LogArea::ID, logarea_hash) != Some(logarea_hash) {
        logarea.draw(layout.get(LogArea::ID));
    }

    io::stdout().flush().ok();
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

impl Rect {
    fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}

fn gen_layout(term_rect: Rect, is_sidemenu_opened: bool) -> Layout {
    let mut term_rect = term_rect;
    let mut layout = vec![];

    let log = Rect {
        y: term_rect.height.saturating_sub(1),
        height: 1.min(term_rect.height),
        ..term_rect
    };

    term_rect = Rect {
        height: term_rect.height.saturating_sub(1),
        ..term_rect
    };

    if is_sidemenu_opened {
        term_rect = Rect {
            x: term_rect.x.saturating_add(20),
            width: term_rect.width.saturating_sub(20),
            ..term_rect
        };

        layout.push(Rect {
            width: 20.min(term_rect.width),
            ..term_rect
        });
    } else {
        layout.push(Rect::empty());
    }

    layout.append(&mut vec![
        Rect {
            height: 1.min(term_rect.height),
            ..term_rect
        },
        Rect {
            y: term_rect.y.saturating_add(1),
            height: 1.min(term_rect.height),
            ..term_rect
        },
        Rect {
            y: term_rect.y.saturating_add(2),
            height: term_rect.height.saturating_sub(3),
            ..term_rect
        },
        Rect {
            y: term_rect.height.saturating_sub(1),
            height: 1.min(term_rect.height),
            ..term_rect
        },
    ]);

    layout.push(log);

    Layout::new(layout)
}

#[macro_export]
macro_rules! log {
    ($($out:expr),+) => {{
        use crossterm::cursor::MoveTo;
        use crossterm::style::ResetColor;
        use crossterm::terminal::{self, Clear, ClearType};

        let Ok((cols, rows)) = terminal::size() else {
            panic!("Couldn't get a tty size");
        };

        print!(
            "{}{} {}{}{}",
            MoveTo(0, rows),
            Clear(ClearType::CurrentLine),
            format_args!($($out),+),
            ResetColor,
            " ".repeat(cols.into())
        )
    }};
}
