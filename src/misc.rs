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

pub fn child_files(path: &PathBuf) -> Vec<PathBuf> {
    if !path.is_dir() || !path.exists() {
        return vec![];
    }

    match path.read_dir() {
        Ok(entries) => entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect(),
        Err(_) => vec![],
    }
}

fn sort_files(files: &mut Vec<PathBuf>) {
    files.sort_by_key(|p| {
        let name = file_name(&p);
        let priority = match name.chars().next() {
            Some('.') => 2,
            Some(c) if c.is_lowercase() => 0,
            Some(c) if c.is_uppercase() => 1,
            _ => 3,
        };
        (priority, name.to_owned())
    });
}

pub fn sorted_child_files(path: &PathBuf) -> Vec<PathBuf> {
    let mut c = child_files(path);
    sort_files(&mut c);
    c
}
