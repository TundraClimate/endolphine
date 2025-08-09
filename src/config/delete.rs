use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct DeleteConfig {
    pub(super) listen_yes: bool,
    pub(super) put_to_temp: bool,
    pub(super) with_yank: bool,
}

impl Default for DeleteConfig {
    fn default() -> Self {
        Self {
            listen_yes: true,
            put_to_temp: false,
            with_yank: false,
        }
    }
}
