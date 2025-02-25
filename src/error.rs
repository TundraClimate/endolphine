use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("Cannot switch Alternate screen")]
    SwitchScreen,

    #[error("Init error")]
    Init(String),

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
        match self {
            Self::SwitchScreen => panic!("Cannot switch Alternate screen"),
            Self::Init(kind) => panic!("Application init failed: {}", kind),
            Self::DisplayViewLine => panic!("Cannot display texts"),
            Self::DisplayMenuLine => panic!("Cannot display texts"),
            Self::Log => panic!("Cant logging texts"),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("Command \"{}\" failed: {}", command, kind))
            }
            Self::Flush(kind) => crate::log!(format!("canvas flush failed: {}", kind)),
        };
    }
}
