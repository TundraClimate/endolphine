use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub(super) struct PasteConfig {
    pub(super) copied_suffix: String,
    pub(super) is_overwrite: bool,
    pub(super) listen_overwrite: bool,
}
