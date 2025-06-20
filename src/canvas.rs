#[macro_export]
macro_rules! log {
    ($($args:expr),+) => {{
        use crossterm::cursor;
        use crossterm::style;
        use crossterm::terminal;
        use crossterm::terminal::ClearType;
        use std::io;
        let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
        crossterm::queue!(
            io::stdout(),
            style::ResetColor,
            cursor::MoveTo(0, row),
            style::Print(format_args!($($args),+)),
            terminal::Clear(ClearType::UntilNewLine)
        )
        .unwrap_or_else(|_| {
            $crate::Error::LogDisplayFailed.handle();
        });
    }};

}

#[macro_export]
macro_rules! dbg_log {
    ($($args:expr),+) => {{
        use crossterm::cursor;
        use crossterm::style;
        use crossterm::terminal;
        use crossterm::terminal::ClearType;
        use std::io;
        let row = terminal::size().map(|(_, h)| h).unwrap_or(100);
        let ts = chrono::Local::now().format("[%H:%M:%S%.3f]").to_string();
        if let Err(_) = crossterm::execute!(
            io::stdout(),
            cursor::MoveTo(0, row),
            style::Print(format!("{} {}", ts, format_args!($($args),+))),
            terminal::Clear(ClearType::UntilNewLine),
        ) {
            $crate::Error::LogDisplayFailed.handle()
        };
    }};
}

pub enum LayoutSpec {
    Min(u16),
    Fill,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn split(&self, specs: Vec<LayoutSpec>, vertical: bool) -> Vec<Rect> {
        let total = if vertical { self.width } else { self.height };
        let mut rem = total;
        let mut dims = vec![0; specs.len()];

        for (i, s) in specs.iter().enumerate() {
            match s {
                LayoutSpec::Min(v) => {
                    dims[i] = *v;
                    rem = rem.saturating_sub(*v);
                }
                LayoutSpec::Fill => {}
            }
        }

        let used: u16 = dims.iter().sum();
        let fills = specs
            .iter()
            .filter(|s| matches!(s, LayoutSpec::Fill))
            .count() as u16;

        for (i, s) in specs.iter().enumerate() {
            if matches!(s, LayoutSpec::Fill) && fills > 0 {
                dims[i] = total.saturating_sub(used) / fills;
            }
        }

        let assigned: u16 = dims.iter().sum();

        if let Some(last) = dims.last_mut() {
            *last += total.saturating_sub(assigned);
        }

        let mut res = vec![];
        let (mut x, mut y) = (self.x, self.y);

        for d in dims {
            res.push(if vertical {
                Self {
                    x,
                    y: self.y,
                    width: d,
                    height: self.height,
                }
            } else {
                Self {
                    x: self.x,
                    y,
                    width: self.width,
                    height: d,
                }
            });

            if vertical {
                x += d;
            } else {
                y += d;
            }
        }
        res
    }

    pub fn split_vertical(&self, specs: Vec<LayoutSpec>) -> Vec<Rect> {
        self.split(specs, true)
    }

    pub fn split_horizontal(&self, specs: Vec<LayoutSpec>) -> Vec<Rect> {
        self.split(specs, false)
    }
}

#[derive(Clone)]
pub struct Canvas {
    rect: Rect,
    default_style: String,
}

impl From<Rect> for Canvas {
    fn from(rect: Rect) -> Self {
        Self {
            rect,
            default_style: String::new(),
        }
    }
}

impl Canvas {
    pub fn print(&self, rel_x: u16, rel_y: u16, s: &str) {
        if self.rect.height <= rel_y || self.rect.width <= rel_x {
            return;
        }

        let abs_x = self.rect.x + rel_x;
        let abs_y = self.rect.y + rel_y;
        let mut text = String::new();
        let mut rem = self.rect.width.saturating_sub(rel_x) as usize;
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
                let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);

                if w > rem {
                    text.push_str(&" ".repeat(rem));
                    break;
                }

                rem -= w;
                text.push(c);
                chars.next();
            }
        }

        crossterm::queue!(
            std::io::stdout(),
            crossterm::cursor::MoveTo(abs_x, abs_y),
            crossterm::style::Print(self.default_style.as_str()),
            crossterm::style::Print(text),
            crossterm::style::ResetColor,
        )
        .ok();
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn fill(&self) {
        for i in 0..self.rect.height {
            self.print(0, i, &" ".repeat(self.rect.width as usize));
        }
    }

    pub fn set_bg(&mut self, bg: crossterm::style::Color) {
        self.default_style
            .push_str(&crossterm::style::SetBackgroundColor(bg).to_string());
    }

    pub fn set_fg(&mut self, fg: crossterm::style::Color) {
        self.default_style
            .push_str(&crossterm::style::SetForegroundColor(fg).to_string());
    }
}
