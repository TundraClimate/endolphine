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

fn sort_files(files: &mut [PathBuf]) {
    files.sort_by_key(|path| {
        let entry_name = entry_name(path);

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
}

pub fn sorted_child_files(path: &Path) -> Vec<PathBuf> {
    let mut child_files = child_files(path);

    sort_files(&mut child_files);

    child_files
}
