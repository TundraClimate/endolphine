use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("cannot switch Alternate screen")]
    SwitchScreen,

    #[error("init error")]
    InitFailed,

    #[error("di_view_line error")]
    DisplayViewLineFailed,

    #[error("di_menu_line error")]
    DisplayMenuLineFailed,

    #[error("logging error")]
    Log,

    #[error("command failed")]
    CommandExecute(String, String),
}

impl EpError {
    pub fn handle(&self, quit_flag: Arc<AtomicBool>) {
        let f = quit_flag.clone();
        let res = match self {
            Self::SwitchScreen => EpError::tui_exit("cannot switch Alternate screen", quit_flag),
            Self::InitFailed => EpError::tui_exit("application init failed", quit_flag),
            Self::DisplayViewLineFailed => EpError::tui_exit("cannot display texts", quit_flag),
            Self::DisplayMenuLineFailed => EpError::tui_exit("cannot display texts", quit_flag),
            Self::Log => EpError::tui_exit("cant logging texts", quit_flag),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("command \"{}\" failed: {}", command, kind))
            }
        };

        if let Err(e) = res {
            e.handle(f);
        }
    }

    fn tui_exit(text: &str, quit_flag: Arc<AtomicBool>) -> EpResult<()> {
        crate::disable_tui!()?;
        quit_flag.swap(true, Ordering::Relaxed);

        eprintln!("app exit: {text}");
        std::process::exit(1);
    }
}
