use crate::global;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

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
    let priority = global::config().sort_by_priority;
    files.sort_by_key(|p| {
        let name = file_name(p);
        let priority = match name.chars().next() {
            Some(c) if c.is_lowercase() => priority[0],
            Some(c) if c.is_uppercase() => priority[1],
            Some('.') => priority[2],
            _ => priority[3],
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

pub fn remove_dir_all(path: &Path) -> std::io::Result<()> {
    let res = WalkDir::new(path)
        .contents_first(true)
        .into_iter()
        .try_for_each(|entry| {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_symlink() || entry_path.is_file() {
                std::fs::remove_file(entry_path)
            } else {
                std::fs::remove_dir(entry_path)
            }
        });

    if matches!(res, Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied) {
        return res;
    }

    if res.is_err() || res.is_ok() && exists_item(path) {
        std::thread::sleep(std::time::Duration::from_millis(100));
        remove_dir_all(path)?;
    }

    Ok(())
}

pub fn into_tmp(paths: &[PathBuf]) -> std::io::Result<()> {
    let tmp_path = Path::new("/tmp").join("endolphine");
    for path in paths {
        if !exists_item(path) {
            continue;
        }

        let dest = tmp_path.join(file_name(path));

        if exists_item(&dest) {
            remove_dir_all(&dest)?;
        }

        std::fs::rename(path, dest)?;
    }

    Ok(())
}
