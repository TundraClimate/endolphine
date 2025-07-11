use crate::state::State;
use std::sync::Arc;

pub fn set_panic_hook() {
    use std::{panic, process};

    panic::set_hook(Box::new(|e| {
        disable();

        if let Some(e) = e.payload().downcast_ref::<String>() {
            terminate(e);
        } else if let Some(e) = e.payload().downcast_ref::<&str>() {
            terminate(e);
        }

        process::exit(1);
    }));
}

pub fn set_dbg_hook() {
    use std::{panic, process};

    panic::set_hook(Box::new(|e| {
        disable();

        if let Some(msg) = e.payload().downcast_ref::<String>() {
            dbg_terminate(msg, e);
        } else if let Some(msg) = e.payload().downcast_ref::<&str>() {
            dbg_terminate(msg, e);
        }

        process::exit(1);
    }));
}

fn terminate<D: std::fmt::Display>(e: D) {
    use crossterm::style::{SetAttribute, SetForegroundColor};

    eprintln!(
        "{}{}",
        SetForegroundColor(crossterm::style::Color::Red),
        SetAttribute(crossterm::style::Attribute::Bold),
    );
    eprintln!("{:-^41}", "Endolphine terminated");
    eprintln!(" {e}");
    eprintln!("{}", "-".repeat(41));
}

fn dbg_terminate<D: std::fmt::Display>(msg: D, e: &std::panic::PanicHookInfo) {
    use crossterm::style::{SetAttribute, SetForegroundColor};

    let location = e.location().unwrap();

    eprintln!(
        "{}{}",
        SetForegroundColor(crossterm::style::Color::Red),
        SetAttribute(crossterm::style::Attribute::Bold),
    );
    eprintln!("{:-^41}", "Endolphine terminated");
    eprintln!(" Cause: {msg}");
    eprintln!(
        " From: '{}' at {}:{}",
        location.file(),
        location.line(),
        location.column()
    );
    eprintln!("{}", "-".repeat(41));
}

pub fn close() -> ! {
    use std::process;

    disable();

    process::exit(0);
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

pub async fn tick_loop<F: Fn(Arc<State>)>(state: Arc<State>, tick_ms: u64, f: F) {
    use tokio::time::{self, Duration, Instant};

    loop {
        let start = Instant::now();

        f(state.clone());

        let elapsed = start.elapsed();

        if elapsed < Duration::from_millis(tick_ms) {
            time::sleep(Duration::from_millis(tick_ms) - elapsed).await;
        }
    }
}
