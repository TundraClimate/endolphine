use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("Cannot switch Alternate screen")]
    SwitchScreen,

    #[error("Init error")]
    Init,

    #[error("di_view_line error")]
    DisplayViewLine,

    #[error("di_menu_line error")]
    DisplayMenuLine,

    #[error("Logging error")]
    Log,

    #[error("Command failed")]
    CommandExecute(String, String),

    #[error("flush error")]
    Flush(String),
}

impl EpError {
    pub fn handle(&self) {
        let res = match self {
            Self::SwitchScreen => {
                eprintln!("Cannot switch Alternate screen");
                std::process::exit(1);
            }
            Self::Init => EpError::tui_exit("Application init failed"),
            Self::DisplayViewLine => EpError::tui_exit("Cannot display texts"),
            Self::DisplayMenuLine => EpError::tui_exit("Cannot display texts"),
            Self::Log => EpError::tui_exit("Cant logging texts"),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("Command \"{}\" failed: {}", command, kind));
                Ok(())
            }
            Self::Flush(kind) => {
                crate::log!(format!("canvas flush failed: {}", kind));
                Ok(())
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
