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
    pub(super) fn sort_func(&self) -> Box<dyn Fn(&mut [PathBuf]) + Send + Sync> {
        use crate::misc;

        Box::new(move |files: &mut [PathBuf]| {
            files.sort_by_key(|path| {
                let entry_name = misc::entry_name(path);

                if &entry_name == ".ep.ed" {
                    return (9999, entry_name.to_owned());
                }

                (
                    match entry_name.chars().next() {
                        Some(c) if c.is_lowercase() => 0,
                        Some(c) if c.is_uppercase() => 1,
                        Some('.') => 2,
                        _ => 3,
                    },
                    entry_name.to_owned(),
                )
            })
        })
    }
}
