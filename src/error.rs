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
            Self::SwitchScreen => panic!("cannot switch Alternate screen"),
            Self::InitFailed => EpError::wrapped_panic("application init failed"),
            Self::DisplayViewLineFailed => EpError::wrapped_panic("cannot display texts"),
            Self::DisplayMenuLineFailed => EpError::wrapped_panic("cannot display texts"),
            Self::Log => EpError::wrapped_panic("cant logging texts"),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("command \"{}\" failed: {}", command, kind))
            }
        };

        if let Err(e) = res {
            e.handle();
        }
    }

    fn wrapped_panic(text: &str) -> EpResult<()> {
        crate::disable_tui!()?;

        panic!("{text}")
    }
}
