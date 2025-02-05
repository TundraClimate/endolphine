use crate::global;
use std::path::{Path, PathBuf};

pub fn file_name(path: &PathBuf) -> &str {
    if path == &PathBuf::from("/") {
        return "";
    }

    path.file_name()
        .map(|o| o.to_str())
        .and_then(|s| s)
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

pub fn next_match_from_search() {
    let cursor = global::cursor();

    let child_files = sorted_child_files(&global::get_path());
    let first_match_pos = child_files[cursor.current() + 1..]
        .iter()
        .chain(child_files[..cursor.current()].iter())
        .position(|f| global::is_match_text(|m| file_name(f).contains(m)))
        .map(|pos| pos + 1)
        .unwrap_or(0);

    cursor.shift_loop(first_match_pos as isize);
}
