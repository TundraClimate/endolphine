use std::path::PathBuf;

pub fn file_name<'a>(path: &'a PathBuf) -> &'a str {
    path.file_name()
        .map(|o| o.to_str())
        .and_then(|s| s)
        .unwrap_or("*Invalid Name*")
}

pub fn parent(path: &PathBuf) -> PathBuf {
    path.parent()
        .map(|p| p.to_path_buf())
        .unwrap_or(PathBuf::from("/"))
}
