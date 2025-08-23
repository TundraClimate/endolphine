use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Default)]
pub(super) struct SortConfig {
    types: Types,
    groups: Groups,
}

impl SortConfig {
    pub(super) fn sort_func(&self) -> Box<dyn Fn(&mut [PathBuf]) + Send + Sync> {
        use crate::misc;

        let ty = self.types;
        let group = self.groups;

        Box::new(move |files| {
            files.sort_by_key(|path| {
                let entry_name = misc::entry_name(path);

                if &entry_name == ".ep.ed" {
                    return (255, 255, entry_name.to_owned());
                }

                (
                    ty.parse_type(path),
                    group.parse_group(&entry_name),
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

#[derive(Deserialize, Serialize, Clone, Copy)]
struct Groups {
    dotfiles: u8,
    first_lower: u8,
    first_upper: u8,
    other: u8,
}

impl Default for Groups {
    fn default() -> Self {
        Self {
            dotfiles: 0,
            first_lower: 1,
            first_upper: 2,
            other: 3,
        }
    }
}

impl Groups {
    fn parse_group(&self, entry_name: &str) -> u8 {
        match entry_name.chars().next() {
            Some(c) if c.is_lowercase() => self.first_lower,
            Some(c) if c.is_uppercase() => self.first_upper,
            Some('.') => self.dotfiles,
            _ => self.other,
        }
    }
}
