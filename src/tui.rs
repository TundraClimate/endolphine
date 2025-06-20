pub fn terminate<D: std::fmt::Display>(e: D) {
    use crossterm::style::{SetAttribute, SetForegroundColor};

    eprintln!(
        "{}{}",
        SetForegroundColor(crossterm::style::Color::Red),
        SetAttribute(crossterm::style::Attribute::Bold),
    );
    eprintln!("{:-^41}", "Endolphine terminated");
    eprintln!(" {}", e);
    eprintln!("{}", "-".repeat(41));
}

pub fn enable() {
    use crossterm::{
        cursor::Hide,
        terminal::{self, DisableLineWrap, EnterAlternateScreen},
    };
    use std::io;

    let _ = terminal::enable_raw_mode().and_then(|_| {
        crossterm::execute!(io::stdout(), EnterAlternateScreen, DisableLineWrap, Hide)
    });
}

pub fn disable() {
    use crossterm::{
        cursor::Show,
        terminal::{self, EnableLineWrap, LeaveAlternateScreen},
    };
    use std::io;

    let _ = terminal::disable_raw_mode().and_then(|_| {
        crossterm::execute!(io::stdout(), LeaveAlternateScreen, Show, EnableLineWrap,)
    });
}
