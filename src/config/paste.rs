use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub(super) struct PasteConfig {
    pub(super) copied_suffix: String,
    pub(super) is_overwrite: bool,
    pub(super) listen_overwrite: bool,
}

impl Default for PasteConfig {
    fn default() -> Self {
        Self {
            copied_suffix: String::from("_COPY"),
            is_overwrite: false,
            listen_overwrite: true,
        }
    }
}
