use super::Exec;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Deserialize, Serialize)]
pub(super) struct EditConfig(BTreeMap<String, String>);

#[derive(Deserialize, Serialize)]
struct HijackInfo {
    cmd: Exec,
    hijack: bool,
}
