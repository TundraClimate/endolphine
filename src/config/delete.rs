use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub(super) struct DeleteConfig {
    pub(super) listen_yes: bool,
    pub(super) with_yank: bool,
    pub(super) put_to_temp: bool,
}
