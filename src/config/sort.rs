use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub(super) struct SortConfig {}

impl Default for SortConfig {
    fn default() -> Self {
        Self {}
    }
}

impl SortConfig {
    pub(super) fn sort_func(&self) -> fn(&mut [PathBuf]) {
        todo!()
    }
}
