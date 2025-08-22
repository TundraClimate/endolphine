use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Default)]
pub(super) struct SortConfig {
    types: Types,
}

impl SortConfig {
    pub(super) fn sort_func(&self) -> Box<dyn Fn(&mut [PathBuf]) + Send + Sync> {
        use crate::misc;

        let ty = self.types;

        Box::new(move |files| {
            files.sort_by_key(|path| {
                let entry_name = misc::entry_name(path);

                if &entry_name == ".ep.ed" {
                    return (255, 9999, entry_name.to_owned());
                }

                (
                    ty.parse_type(path),
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

#[derive(Deserialize, Serialize, Clone, Copy, Default)]
struct Types {
    file: u8,
    directory: u8,
    symlink_file: u8,
    symlink_dir: u8,
    other: u8,
}

impl Types {
    fn parse_type(&self, path: &Path) -> u8 {
        match path {
            path if path.is_symlink() && path.is_file() => self.symlink_file,
            path if path.is_symlink() && path.is_dir() => self.symlink_dir,
            path if path.is_dir() => self.directory,
            path if path.is_file() => self.file,
            _ => self.other,
        }
    }
}
