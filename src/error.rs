use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("Cannot switch screen modes")]
    SwitchScreen,

    #[error("Initialization of the application failed: {0}")]
    Init(String),

    #[error("The display of the screen lines failed")]
    Display,

    #[error("Command execution failed")]
    CommandExecute(String, String),

    #[error("Terminal flush failed")]
    Flush(String),
}

impl EpError {
    pub fn handle(&self) {
        match self {
            Self::SwitchScreen => panic!("{}", self),
            Self::Init(_) => panic!("{}", self),
            Self::Display => panic!("{}", self),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("Failed to run \"{}\": {}", command, kind))
            }
            Self::Flush(kind) => crate::log!(format!("Failed canvas flush: {}", kind)),
        };
    }
}
