use crate::state::State;
use flexi_logger::FlexiLoggerError;
use std::{io, path::PathBuf, sync::Arc};

fn local_path() -> PathBuf {
    use std::path::Path;

    let Some(home) = option_env!("HOME") else {
        panic!("Couldn't read the $HOME");
    };

    Path::new(home)
        .join(".local")
        .join("share")
        .join("endolphine")
}

pub fn setup_local() -> io::Result<()> {
    use std::{fs, path::Path};

    let tmp_dir = Path::new("/tmp").join("endolphine");

    if !tmp_dir.exists() {
        log::info!("Temp directory is couldn't find");

        log::info!("Create the temp dir...");
        fs::create_dir_all(&tmp_dir)?;
        log::info!("The temp dir successfully created");
    }

    let local_cb = tmp_dir.join("cb.txt");

    if !local_cb.exists() {
        log::info!("Local clipboard is couldn't find");

        log::info!("Create the local clipboard...");
        fs::write(local_cb, b"")?;
        log::info!("The local clipboard successfully created");
    }

    let trash = tmp_dir.join("Trash");

    if !trash.exists() {
        log::info!("App trash is couldn't find");

        log::info!("Create the app trash...");
        fs::create_dir_all(&trash)?;
        log::info!("The app trash successfully created");
    }

    Ok(())
}

pub fn setup_logger() -> Result<(), FlexiLoggerError> {
    use flexi_logger::{Age, Cleanup, Criterion, FileSpec, Logger, Naming};

    Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .directory(local_path().join("log"))
                .suppress_basename()
                .suffix("log"),
        )
        .append()
        .rotate(
            Criterion::Age(Age::Day),
            Naming::TimestampsCustomFormat {
                current_infix: None,
                format: "%Y-%m-%d",
            },
            Cleanup::KeepLogFiles(5),
        )
        .format_for_files(|w, now, record| {
            write!(
                w,
                "[{}] [{}] {}",
                now.format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .start()
        .map(|_| ())
}

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

    log::error!("{e}");
    log::error!("\n---\nEndolphine terminated\n---");

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

    log::error!("{msg}");
    log::error!(
        "Termination from '{}' at {}:{}",
        location.file(),
        location.line(),
        location.column()
    );
    log::error!("\n---\nEndolphine terminated\n---");

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

    log::info!("Endolphine closed");

    process::exit(0);
}

pub fn update_title<D: AsRef<str>>(title: D) {
    use crossterm::terminal::SetTitle;
    use std::io;

    crossterm::execute!(io::stdout(), SetTitle(title.as_ref())).ok();
}

pub fn enable() {
    use crossterm::{
        cursor::Hide,
        terminal::{self, DisableLineWrap, EnterAlternateScreen},
    };
    use std::io;

    let _ = terminal::enable_raw_mode().and_then(|_| {
        log::info!("Enter alternate screen");
        crossterm::execute!(io::stdout(), EnterAlternateScreen, DisableLineWrap, Hide)
    });
}

pub fn disable() {
    use crossterm::{
        cursor::Show,
        terminal::{self, EnableLineWrap, LeaveAlternateScreen, SetTitle},
    };
    use std::io;

    let _ = terminal::disable_raw_mode().and_then(|_| {
        log::info!("Leave alternate screen");
        crossterm::execute!(
            io::stdout(),
            LeaveAlternateScreen,
            Show,
            EnableLineWrap,
            SetTitle("")
        )
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
