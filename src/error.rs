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
    pub fn handle(&self) {
        let res = match self {
            Self::SwitchScreen => {
                eprintln!("cannot switch Alternate screen");
                std::process::exit(1);
            }
            Self::InitFailed => EpError::tui_exit("application init failed"),
            Self::DisplayViewLineFailed => EpError::tui_exit("cannot display texts"),
            Self::DisplayMenuLineFailed => EpError::tui_exit("cannot display texts"),
            Self::Log => EpError::tui_exit("cant logging texts"),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("command \"{}\" failed: {}", command, kind))
            }
        };

        if let Err(e) = res {
            e.handle();
        }
    }

    fn tui_exit(text: &str) -> EpResult<()> {
        crate::disable_tui!()?;

        eprintln!("app exit: {text}");
        std::process::exit(1);
    }
}
