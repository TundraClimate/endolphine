use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("Cannot switch screen modes")]
    SwitchScreen,

    #[error("Initialization of the application failed: {0}")]
    Init(String),

    #[error("Command execution failed")]
    CommandExecute(String, String),
}

impl EpError {
    pub fn handle(&self) {
        match self {
            Self::SwitchScreen => panic!("{}", self),
            Self::Init(_) => panic!("{}", self),
            Self::CommandExecute(command, kind) => {
                crate::log!(format!("Failed to run \"{}\": {}", command, kind))
            }
        };
    }
}
