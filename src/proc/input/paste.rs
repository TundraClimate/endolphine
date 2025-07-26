use crate::state::State;
use std::{io, path::Path, sync::Arc};

pub fn ask_paste(state: Arc<State>) {
    super::input_start(&state, "PasteFromCb");
    crate::log!("Overwrite a file (Y/n): ")
}

pub(super) fn restore_paste(state: Arc<State>) {
    use crate::proc::view;

    view::refresh(state.clone());
}

pub(super) fn complete_paste(state: &State, content: &str) {
    let overwrite = content.to_ascii_lowercase().starts_with("y");

    match paste_from_cb(&state.work_dir.get(), overwrite) {
        Ok(count) => crate::log!("{count} items paste successful"),
        Err(e) => crate::log!("Failed to paste from the clipboard: {}", e.kind()),
    }
}

fn paste_from_cb(dir: &Path, overwrite: bool) -> io::Result<usize> {
    use crate::{clipboard, config, misc};

    let config = config::get();

    let files = if config.native_cb {
        if !clipboard::is_cmd_installed() {
            crate::log!(
                "Failed to paste from the clipboard: {} is not installed",
                clipboard::command()
            );

            return Ok(0);
        }

        clipboard::read_native("text/uri-list")
    } else {
        clipboard::read()
    }?;

    let files = files
        .lines()
        .filter_map(|s| s.strip_prefix("file://"))
        .map(Path::new)
        .filter(|path| path.symlink_metadata().is_ok_and(|meta| meta.is_symlink()) || path.exists())
        .collect::<Vec<_>>();

    let count = files
        .iter()
        .map(|from| {
            let to = dir.join(misc::entry_name(from));

            copy_item(*from, &to, overwrite)
        })
        .sum::<usize>();

    Ok(count)
}

fn copy_item<P: AsRef<Path>>(from: P, to: P, overwrite: bool) -> usize {
    use crate::misc;
    use std::{fs, os::unix};
    use walkdir::WalkDir;

    let from = from.as_ref();
    let mut to = to.as_ref().to_path_buf();

    if from == to {
        let mut entry = misc::entry_name(from);
        let parent = from.parent().unwrap_or(Path::new("/"));

        recursive_suffix(parent, &mut entry, "_COPY");

        to = parent.join(entry);
    }

    let mut counter = 0usize;
    let is_to_exists = to.symlink_metadata().is_ok_and(|meta| meta.is_symlink()) || to.exists();

    match from {
        from if from.is_symlink() => {
            if let Ok(origin) = from.read_link()
                && (!is_to_exists || overwrite)
                && unix::fs::symlink(origin, &to).is_ok()
            {
                counter += 1;
            }
        }
        from if from.is_dir() => {
            for entry in WalkDir::new(from).into_iter().flatten() {
                let Ok(rel_path) = entry.path().strip_prefix(from) else {
                    continue;
                };

                let from = entry.path();

                let to = to.join(rel_path);
                let is_to_exists =
                    to.symlink_metadata().is_ok_and(|meta| meta.is_symlink()) || to.exists();

                if is_to_exists && !overwrite {
                    continue;
                }

                if let Some(parent) = to.parent()
                    && !parent.exists()
                    && let Err(_) = fs::create_dir_all(parent)
                {
                    continue;
                }

                if from.is_symlink() {
                    if let Ok(from) = from.read_link()
                        && unix::fs::symlink(from, &to).is_ok()
                    {
                        counter += 1;
                    }
                } else if from.is_dir() {
                    if fs::create_dir_all(&to).is_ok() {
                        counter += 1;
                    }
                } else if fs::copy(from, &to).is_ok() {
                    counter += 1;
                }
            }
        }
        from => {
            if (!is_to_exists || overwrite) && fs::copy(from, &to).is_ok() {
                counter += 1;
            }
        }
    }

    counter
}

fn recursive_suffix(dir: &Path, entry: &mut String, suffix: &str) {
    let path = dir.join(&entry);

    if path.symlink_metadata().is_ok_and(|meta| meta.is_symlink()) || path.exists() {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(entry.to_owned());
        let extension = path
            .extension()
            .map(|s| format!(".{}", s.to_string_lossy()))
            .unwrap_or("".to_string());

        *entry = format!("{stem}{suffix}{extension}");

        recursive_suffix(dir, entry, suffix);
    }
}
