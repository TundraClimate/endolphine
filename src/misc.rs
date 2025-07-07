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
