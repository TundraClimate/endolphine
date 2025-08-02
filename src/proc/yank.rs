use crate::state::State;
use std::{io, path::Path, sync::Arc};

pub fn clip_files<P: AsRef<Path>>(files: &[P]) -> io::Result<()> {
    use crate::{clipboard, config};

    let files = files
        .iter()
        .map(|p| format!("file://{}\n", p.as_ref().to_string_lossy()))
        .collect::<String>();

    let conf = config::get();

    if conf.native_cb {
        clipboard::clip_native(files, "text/uri-list")
    } else {
        clipboard::clip(files)
    }
}

pub fn yank(state: Arc<State>) {
    use crate::{config, misc};

    let child_files = misc::sorted_child_files(&state.work_dir.get());

    if let Some(target) = child_files.get(state.file_view.cursor.current()) {
        let native = config::get().native_cb;

        if native {
            log::info!("Clipping by native command");
        } else {
            log::info!("Clipping by endolphine");
        }

        match clip_files(&[target]) {
            Ok(_) => {
                log::info!("Now clipboard: \n{}", target.to_string_lossy());
                crate::log!("Yanked '{}'", misc::entry_name(target));
            }
            Err(e) => {
                log::warn!("Clip the '{}' is failed", target.to_string_lossy());
                log::warn!("Failed kind: {}", e.kind());
                crate::log!(
                    "Failed to yank the '{}': {}",
                    target.to_string_lossy(),
                    e.kind()
                );
            }
        }
    }
}

pub fn yank_selects(state: Arc<State>) {
    use crate::{config, misc, proc::view};

    let child_files = misc::sorted_child_files(&state.work_dir.get());
    let selected = state.file_view.selection.collect();

    let targets = child_files
        .into_iter()
        .enumerate()
        .filter_map(|(i, c)| selected.contains(&i).then_some(c))
        .collect::<Vec<_>>();

    let native = config::get().native_cb;

    if native {
        log::info!("Clipping by native command");
    } else {
        log::info!("Clipping by endolphine");
    }

    match clip_files(&targets) {
        Ok(_) => {
            log::info!("Now clipboard: \n{targets:?}");
            crate::log!("Yanked {} items", targets.len());
        }
        Err(e) => {
            log::warn!("Clip files is failed");
            log::warn!(
                "\n{}",
                toml::to_string_pretty(&targets).unwrap_or("Files cannot read".to_string())
            );
            log::warn!("Failed kind: {}", e.kind());
            crate::log!("Failed to yank {} files: {}", targets.len(), e.kind());
        }
    }

    if !selected.is_empty() {
        view::initialize(&state);

        let cursor = &state.file_view.cursor;
        let first = selected[0];

        cursor.reset();
        cursor.shift_p(first);
    }
}
