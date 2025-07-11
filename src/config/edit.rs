use super::Exec;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

#[derive(Deserialize, Serialize)]
pub(super) struct EditConfig(BTreeMap<String, HijackInfo>);

#[derive(Deserialize, Serialize)]
pub struct HijackInfo {
    pub cmd: Exec,
    pub hijack: bool,
}

pub struct HijackMapping(BTreeMap<String, HijackInfo>);

impl HijackMapping {
    pub(super) fn new(config: Option<EditConfig>) -> Self {
        match config {
            Some(config) => Self(config.0),
            None => Self(BTreeMap::new()),
        }
    }

    pub fn get(&self, file: &Path) -> Option<&HijackInfo> {
        if !file.is_file() {
            return None;
        }

        let extension = file.extension()?.to_str()?;

        self.0.get(extension)
    }
}
