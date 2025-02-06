use crate::global;
use std::path::{Path, PathBuf};

pub fn file_name(path: &Path) -> &str {
    if path == Path::new("/") {
        return "";
    }

    path.file_name()
        .and_then(|o| o.to_str())
        .unwrap_or("_OsIncompatible_")
}

pub fn parent(path: &Path) -> PathBuf {
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(PathBuf::from("/"))
}

pub fn child_files(path: &Path) -> Vec<PathBuf> {
    if !path.is_dir() || !path.exists() {
        return vec![];
    }

    match path.read_dir() {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => vec![],
    }
}

fn sort_files(files: &mut [PathBuf]) {
    files.sort_by_key(|p| {
        let name = file_name(p);
        let priority = match name.chars().next() {
            Some('.') => 2,
            Some(c) if c.is_lowercase() => 0,
            Some(c) if c.is_uppercase() => 1,
            _ => 3,
        };
        (priority, name.to_owned())
    });
}

pub fn sorted_child_files(path: &Path) -> Vec<PathBuf> {
    let mut c = child_files(path);
    sort_files(&mut c);
    c
}

pub fn child_files_len(path: &Path) -> usize {
    if !path.is_dir() || !path.exists() {
        return 0;
    }

    match path.read_dir() {
        Ok(d) => d.count(),
        Err(_) => 0,
    }
}

pub fn body_height() -> u16 {
    global::get_height().saturating_sub(4)
}

pub fn exists_item(path: &Path) -> bool {
    path.symlink_metadata()
        .is_ok_and(|m| m.is_symlink() || path.exists())
}
