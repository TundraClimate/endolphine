#[macro_export]
macro_rules! global {
    (static $name:ident : $type:ty = $init:expr;) => {
        static $name: std::sync::LazyLock<$type> = std::sync::LazyLock::new(|| $init);
    };
}

#[macro_export]
macro_rules! sys_log {
    ($lv:expr, $($fmt:expr),+) => {{
        let now = chrono::Local::now();
        let output_path = std::path::Path::new(option_env!("HOME").unwrap_or("/root"))
            .join(".local")
            .join("share")
            .join("endolphine")
            .join("log")
            .join(now.format("%Y_%m_%d.log").to_string());
        let log_header = now.format("[%H:%M:%S]").to_string();
        let log_lvl = match $lv.to_ascii_lowercase().as_str() {
            "warn" | "w" => "[WARN]",
            "error" | "err" | "e" => "[ERROR]",
            "info" | "i" => "[INFO]",
            _ => "[INFO]",
        };
        let fmt_txt = format!("\n{} {} {}", log_header, log_lvl, format_args!($($fmt),+));

        use std::io::Write;

        let mut output_file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_path)
            .map_err(|e| $crate::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
        output_file
            .write_all(fmt_txt.as_bytes())
            .map_err(|e| $crate::Error::OutLogToFileFailed(e.kind().to_string()))
            .unwrap();
    }};
}

pub fn enable_tui() -> Result<(), crate::Error> {
    crossterm::terminal::enable_raw_mode()
        .and_then(|_| {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::cursor::Hide,
                crossterm::terminal::DisableLineWrap,
            )
        })
        .map_err(|_| crate::Error::ScreenModeChangeFailed)
}

pub fn disable_tui() -> Result<(), crate::Error> {
    crossterm::terminal::disable_raw_mode()
        .and_then(|_| {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::cursor::Show,
                crossterm::terminal::EnableLineWrap,
            )
        })
        .map_err(|_| crate::Error::ScreenModeChangeFailed)
}

pub async fn launch(path: &std::path::Path) -> Result<(), crate::Error> {
    if !crate::misc::exists_item(path) || path.is_file() {
        sys_log!("e", "Invalid input detected");

        return Err(crate::Error::InvalidArgument(format!(
            "invalid path (-> {})",
            path.to_string_lossy()
        )));
    }

    enable_tui()?;

    sys_log!(
        "i",
        "Endolphine launch in {} successfully",
        path.canonicalize()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(path.to_string_lossy().to_string())
    );

    if crate::config::check().is_err() {
        crate::log!("Failed load config.toml, use the Default config");
    }

    let cp_handle = tokio::spawn(async move { components().await });

    cp_handle.await.unwrap();

    disable_tui()?;

    Ok(())
}

async fn components() {
    let components = crate::component::components();

    if let Err(e) = components.on_init() {
        e.handle();
    }

    if let Err(e) = on_tick(&*components).await {
        e.handle();
    }
}

async fn on_tick(components: &dyn crate::component::Component) -> Result<(), crate::Error> {
    loop {
        let start = tokio::time::Instant::now();

        components.on_tick()?;

        let elapsed = start.elapsed();
        let tick = 70;
        if elapsed < tokio::time::Duration::from_millis(tick) {
            tokio::time::sleep(tokio::time::Duration::from_millis(tick) - elapsed).await;
        }
    }
}
