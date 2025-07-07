use std::path::Path;

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
