use std::path::{Path, PathBuf};

pub fn entry_name(path: &Path) -> String {
    if path == Path::new("/") {
        return String::from("");
    }

    match path
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
    {
        Some(name) => name,
        None => path.to_string_lossy().to_string(),
    }
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

pub fn child_files(path: &Path) -> Vec<PathBuf> {
    if !path.is_dir() || !path.exists() {
        return vec![];
    }

    match path.read_dir() {
        Ok(entries) => entries.flatten().map(|entry| entry.path()).collect(),
        Err(_) => vec![],
    }
}

pub fn sorted_child_files(path: &Path) -> Vec<PathBuf> {
    use crate::config;

    let mut child_files = child_files(path);
    let config = config::get();

    (config.sort_func)(&mut child_files);

    child_files
}
