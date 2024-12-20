use thiserror::Error;

pub type EpResult<T> = Result<T, EpError>;

#[derive(Error, Debug)]
pub enum EpError {
    #[error("cannot switch Alternate screen")]
    SwitchScreen,

    #[error("init error")]
    InitFailed,
}

impl EpError {
    pub fn handle(&self) {
        match self {
            Self::SwitchScreen => panic!("cannot switch Alternate screen"),
            Self::InitFailed => panic!("application init failed"),
        }
    }
}
