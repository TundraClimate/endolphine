use super::Command;
use super::Component;
use super::app::AppState;
use super::root::RootState;

#[derive(Default)]
struct Selection {
    inner: Option<(usize, usize)>,
}

impl Selection {
    fn is_active(&self) -> bool {
        self.inner.is_some()
    }

    fn enable(&mut self, base_pos: usize) {
        self.inner = Some((base_pos, base_pos));
    }

    fn disable(&mut self) {
        self.inner = None;
    }

    fn select_area(&mut self, other: usize) {
        if let Some((base, _)) = self.inner {
            self.inner = Some((base, other));
        }
    }

    fn select_if_active(&mut self, pos: usize) {
        if self.is_active() {
            self.select_area(pos);
        }
    }

    fn is_selected(&self, i: usize) -> bool {
        if !self.is_active() {
            return false;
        }

        if let Some((base, pin)) = self.inner {
            let min = base.min(pin);
            let max = base.max(pin);
            (min..=max).contains(&i)
        } else {
            false
        }
    }
}

#[derive(Default)]
struct EpGrep {
    inner: String,
}

impl EpGrep {
    fn reset(&mut self) {
        self.inner.clear();
    }

    fn is_match_found(&self, target: &str) -> bool {
        regex::Regex::new(&self.inner).is_ok_and(|regex| regex.is_match(target))
    }
}

#[derive(Default)]
pub struct BodyState {
    cursor: crate::cursor::Cursor,
    selection: Selection,
    pub input: crate::input::Input,
    grep: EpGrep,
}

pub struct Body {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
    inner: Vec<Box<dyn Component>>,
}

impl Body {
    pub fn with_state<
        F: FnOnce(std::sync::Arc<std::sync::RwLock<BodyState>>) -> Vec<Box<dyn Component>>,
    >(
        app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
        root_state: std::sync::Arc<std::sync::RwLock<RootState>>,
        f: F,
    ) -> Self {
        let body_state = std::sync::Arc::new(std::sync::RwLock::new(BodyState::default()));

        Self {
            state: body_state.clone(),
            app_state,
            root_state,
            inner: f(body_state.clone()),
        }
    }
}

struct MoveDown {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for MoveDown {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();

        state.cursor.shift_p(self.prenum);

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

struct MoveUp {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for MoveUp {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();

        state.cursor.shift_n(self.prenum);

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

struct MoveTop {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
}

impl Command for MoveTop {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();

        state.cursor.reset();

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

struct MoveBottom {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
}

impl Command for MoveBottom {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();
        let len = state.cursor.len();

        state.cursor.shift_p(len);

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

struct PageDown {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for PageDown {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();
        let page_len = crate::misc::body_height() as usize;

        state.cursor.shift_p(self.prenum * page_len);

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

struct PageUp {
    state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    prenum: usize,
}

impl Command for PageUp {
    fn run(&self) -> Result<(), crate::Error> {
        let mut state = self.state.write().unwrap();
        let page_len = crate::misc::body_height() as usize;

        state.cursor.shift_n(self.prenum * page_len);

        let cursor_pos = state.cursor.current();
        state.selection.select_if_active(cursor_pos);

        Ok(())
    }
}

fn move_current_dir(
    app_state: &mut AppState,
    body_state: &mut BodyState,
    path: &std::path::Path,
) -> Result<(), crate::Error> {
    body_state.selection.disable();
    app_state.path.swap(path)?;

    crate::sys_log!("i", "Change the open directory: {}", path.to_string_lossy());

    let cursor = &mut body_state.cursor;

    cursor.resize(crate::misc::child_files_len(path));
    cursor.reset();

    Ok(())
}

struct MoveParent {
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
}

impl Command for MoveParent {
    fn run(&self) -> Result<(), crate::Error> {
        let path = self.app_state.read().unwrap().path.get().clone();

        if path == std::path::Path::new("/") {
            return Ok(());
        }

        let mut app_state = self.app_state.write().unwrap();
        let mut body_state = self.body_state.write().unwrap();
        let old_child_files = crate::misc::sorted_child_files(&path);
        let old_cursor_pos = body_state.cursor.current();
        let parent = crate::misc::parent(&path);

        move_current_dir(&mut app_state, &mut body_state, &parent)?;

        let child_files = crate::misc::sorted_child_files(&parent);
        let cursor = &mut body_state.cursor;

        if let Some(target_path) = old_child_files.get(old_cursor_pos) {
            let mut cur = cursor.cache.write().unwrap();
            cur.wrap_node(target_path);
        }

        if let Some(pos) = child_files.into_iter().position(|p| p == path) {
            cursor.shift_p(pos);
        }

        Ok(())
    }
}

struct EnterDirOrEdit {
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
}

impl Command for EnterDirOrEdit {
    fn run(&self) -> Result<(), crate::Error> {
        let path = self.app_state.read().unwrap().path.get().clone();
        let child_files = crate::misc::sorted_child_files(&path);

        if child_files.is_empty() {
            return Ok(());
        }

        let target_path = {
            let body = self.body_state.read().unwrap();
            let cursor = &body.cursor;

            let Some(target_path) = child_files.get(cursor.current()) else {
                return Ok(());
            };

            target_path
        };

        if target_path.is_dir() {
            let mut app = self.app_state.write().unwrap();
            let mut body = self.body_state.write().unwrap();

            move_current_dir(&mut app, &mut body, target_path)?;

            let cursor = &body.cursor;
            let mut cache = cursor.cache.write().unwrap();

            if let Some(pos) = child_files.iter().position(|e| cache.inner_equal(e)) {
                cursor.shift_p(pos);
                cache.unwrap_surface();
            } else {
                cache.reset();
            }
        } else {
            let body = self.app_state.read().unwrap();
            let config = body.config.get();
            let mut cmd = config.editor.clone();
            let mut in_term = true;

            if let Some(extension) = target_path.extension().map(|e| e.to_string_lossy()) {
                if let Some(opts) = config
                    .open
                    .as_ref()
                    .and_then(|opt| opt.corresponding_with(&extension))
                {
                    cmd = opts.cmd;
                    in_term = opts.in_term.unwrap_or(true);

                    crate::sys_log!("i", "Override open command: {}", cmd.join(" "));
                }
            }

            let Some((cmd, args)) = cmd.split_first() else {
                crate::sys_log!("w", "Invalid config: open command is empty");
                crate::log!("Invalid config: editor or opener");

                return Ok(());
            };

            if in_term {
                crate::app::disable_tui()?;
            }

            crate::sys_log!(
                "i",
                "Open file with {}: {}",
                cmd,
                target_path.to_string_lossy()
            );

            std::process::Command::new(cmd)
                .args(args)
                .arg(target_path)
                .status()
                .map_err(|e| {
                    crate::Error::CommandExecutionFailed(cmd.to_owned(), e.kind().to_string())
                })?;

            if in_term {
                crate::app::enable_tui()?;
                /* canvas::cache_clear(); */
            }
        }

        Ok(())
    }
}

struct VisualSelect {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for VisualSelect {
    fn run(&self) -> Result<(), crate::Error> {
        use super::app::Mode;

        let mut app = self.app_state.write().unwrap();
        let mut body = self.body_state.write().unwrap();
        let cursor_pos = body.cursor.current();
        let selection = &mut body.selection;

        if selection.is_active() {
            selection.disable();
            app.mode = Mode::Normal;
        } else {
            selection.enable(cursor_pos);
            app.mode = Mode::Visual;
        }

        Ok(())
    }
}

struct CreateFileOrDir {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    content: String,
    is_file: bool,
}

impl Command for CreateFileOrDir {
    fn run(&self) -> Result<(), crate::Error> {
        let path = self
            .app_state
            .read()
            .unwrap()
            .path
            .get()
            .join(&self.content);

        if path.exists() {
            crate::sys_log!(
                "w",
                "Command CreateFileOrDir failed: \"{}\" is already exists",
                self.content
            );
            crate::log!(
                "Add new file failed: \"{}\" is already exists",
                self.content
            );

            return Ok(());
        }

        let add_res = if self.is_file {
            std::fs::write(&path, "")
        } else {
            std::fs::create_dir(&path)
        };

        if let Err(e) = add_res {
            crate::sys_log!("w", "Command CreateFileOrDir failed: {}", e.kind());
            crate::log!("Add new file failed: {}", e.kind());

            return Ok(());
        }

        self.body_state
            .read()
            .unwrap()
            .cursor
            .resize(crate::misc::child_files_len(&path));
        crate::sys_log!(
            "w",
            "Command CreateFileOrDir successful: create the {}",
            self.content
        );
        crate::log!("\"{}\" create successful", self.content);

        Ok(())
    }
}

struct AskCreate {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for AskCreate {
    fn run(&self) -> Result<(), crate::Error> {
        let mut body_state = self.body_state.write().unwrap();

        body_state.selection.disable();
        body_state.input.enable("", Some("CreateFileOrDir".into()));

        self.app_state.write().unwrap().mode = super::app::Mode::Input;

        crate::sys_log!("i", "Called command: CreateFileOrDir");
        crate::log!("Enter name for new File or Directory (for Directory, end with \"/\")");

        Ok(())
    }
}

struct DeleteFileOrDir {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for DeleteFileOrDir {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let path = app_state.path.get().clone();
        let body_state = self.body_state.read().unwrap();
        let cursor = &body_state.cursor;
        let child_files = crate::misc::sorted_child_files(&path);
        let Some(under_cursor_file) = child_files.get(cursor.current()) else {
            crate::sys_log!(
                "w",
                "Command DeleteFileOrDir failed: cursor in invalid position"
            );
            crate::log!("Delete file failed: target cannot find");

            return Ok(());
        };

        let Ok(metadata) = under_cursor_file.symlink_metadata() else {
            crate::sys_log!(
                "w",
                "Command DeleteFileOrDir failed: target metadata cannot access"
            );
            crate::log!("Delete file failed: cannot access metadata");

            return Ok(());
        };

        if !under_cursor_file.exists() && !metadata.is_symlink() {
            crate::sys_log!(
                "w",
                "Command DeleteFileOrDir failed: target file not exists"
            );
            crate::log!("Delete file failed: target not exists");

            return Ok(());
        }

        let config = app_state.config.get();
        let is_alt_for_tmp = config.delete.for_tmp;
        let is_yank = config.delete.yank;
        let is_native_clip = is_yank && config.native_clip;

        let res = if is_alt_for_tmp {
            if is_yank {
                let tmp_dir = std::path::Path::new("/tmp").join("endolphine");
                let target_path = tmp_dir.join(crate::misc::file_name(under_cursor_file));

                crate::misc::clip_paths(is_native_clip, &[target_path]);
            }

            crate::misc::into_tmp(&[under_cursor_file.to_path_buf()])
        } else if under_cursor_file.is_dir() {
            crate::misc::remove_dir_all(under_cursor_file)
        } else {
            std::fs::remove_file(under_cursor_file)
        };

        if let Err(e) = res {
            crate::sys_log!("w", "Command DeleteFileOrDir failed: {}", e.kind());
            crate::log!("Delete file failed: {}", e.kind());

            return Ok(());
        }

        cursor.resize(crate::misc::child_files_len(&path));

        let name = crate::misc::file_name(under_cursor_file);

        crate::sys_log!(
            "i",
            "Command DeleteFileOrDir successful: delete the \"{}\"",
            name
        );
        crate::log!("\"{}\" delete successful", name);

        Ok(())
    }
}

struct DeleteSelected {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for DeleteSelected {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let path = app_state.path.get().clone();
        let selected = {
            let body_state = self.body_state.read().unwrap();
            crate::misc::sorted_child_files(&path)
                .into_iter()
                .enumerate()
                .filter_map(|(i, f)| body_state.selection.is_selected(i).then_some(f))
                .collect::<Vec<_>>()
        };
        let config = app_state.config.get();
        let is_alt_for_tmp = config.delete.for_tmp;
        let is_yank = config.delete.yank;
        let is_native_clip = is_yank && config.native_clip;

        if is_alt_for_tmp {
            if is_yank {
                let tmp_dir = std::path::Path::new("/tmp").join("endolphine");
                let target_paths = selected
                    .iter()
                    .map(|item| tmp_dir.join(crate::misc::file_name(item)))
                    .collect::<Vec<_>>();

                crate::misc::clip_paths(is_native_clip, &target_paths);
            }

            if let Err(e) = crate::misc::into_tmp(&selected) {
                crate::sys_log!("w", "Command DeleteSelected failed: {}", e.kind());
                crate::log!("Delete file failed: {}", e.kind());

                return Ok(());
            }
        } else {
            for target in &selected {
                let Ok(metadata) = target.symlink_metadata() else {
                    crate::sys_log!(
                        "w",
                        "Command DeleteSelected failed: target metadata cannot access"
                    );
                    crate::log!("Delete file failed: cannot access metadata");

                    return Ok(());
                };

                if !target.exists() && !metadata.is_symlink() {
                    crate::sys_log!("w", "Command DeleteSelected failed: target file not exists");
                    crate::log!("Delete file failed: target not exists");

                    return Ok(());
                }

                let res = if target.is_dir() {
                    crate::misc::remove_dir_all(target)
                } else {
                    std::fs::remove_file(target)
                };

                if let Err(e) = res {
                    crate::sys_log!("w", "Command DeleteSelected failed: {}", e.kind());
                    crate::log!("Delete file failed: {}", e.kind());

                    return Ok(());
                }
            }
        }

        let mut body_state = self.body_state.write().unwrap();

        body_state
            .cursor
            .resize(crate::misc::child_files_len(&path));
        body_state.selection.disable();
        crate::sys_log!(
            "i",
            "Command DeleteSelected successful: {} files deleted",
            selected.len()
        );
        crate::log!("{} items delete successful", selected.len());

        Ok(())
    }
}

struct AskDelete {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for AskDelete {
    fn run(&self) -> Result<(), crate::Error> {
        let mut body_state = self.body_state.write().unwrap();
        let under_cursor_file =
            crate::misc::sorted_child_files(self.app_state.read().unwrap().path.get())
                .get(body_state.cursor.current())
                .cloned();
        let is_selection = body_state.selection.is_active();
        let selected = body_state.selection.inner;
        let input = &mut body_state.input;

        if is_selection {
            input.enable("", Some("DeleteSelected".into()));
            crate::sys_log!("i", "Called command: DeleteSelected");

            let selected_len = selected
                .map(|(base, pin)| {
                    let max = base.max(pin);
                    let min = base.min(pin);

                    (min..=max).count()
                })
                .unwrap();

            crate::log!("Delete {} items ? (y/Y)", selected_len);
        }

        if let Some(under_cursor_file) = under_cursor_file {
            input.enable("", Some("DeleteFileOrDir".into()));
            crate::sys_log!("i", "Called command: DeleteFileOrDir");
            crate::log!(
                "Delete \"{}\" ? (y/Y)",
                crate::misc::file_name(&under_cursor_file)
            );
        }

        Ok(())
    }
}

struct Paste {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for Paste {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let config = &app_state.config.get();
        let native_clip = config.native_clip;

        let files = if native_clip {
            if !crate::clipboard::is_cmd_installed() {
                crate::sys_log!(
                    "w",
                    "File paste failed: native command not installed, and config the native-clip is enabled"
                );
                crate::log!("Paste failed: command not installed (ex: wl-paste, xclip)");

                return Ok(());
            }

            match crate::clipboard::read_clipboard_native("text/uri-list") {
                Ok(text) => text
                    .lines()
                    .filter_map(|f| f.strip_prefix("file://"))
                    .map(std::path::PathBuf::from)
                    .filter(|f| crate::misc::exists_item(f))
                    .collect::<Vec<std::path::PathBuf>>(),

                Err(e) => {
                    crate::sys_log!("w", "Couldn't read a clipboard: {}", e.kind());
                    crate::log!("Paste failed: {}", e.kind());

                    return Ok(());
                }
            }
        } else {
            crate::clipboard::read_clipboard()
                .split('\n')
                .map(std::path::PathBuf::from)
                .filter(|c| crate::misc::exists_item(c))
                .collect::<Vec<_>>()
        };

        let current_path = app_state.path.get();
        let suffix = config.paste.similar_file_suffix();
        let is_overwrite = config.paste.default_overwrite;

        for file in files.iter() {
            let Ok(metadata) = file.symlink_metadata() else {
                continue;
            };

            if !crate::misc::exists_item(file) {
                continue;
            }

            let copied_path = current_path.join(crate::misc::file_name(file));
            let copied_path = if copied_path == *file {
                let stem = copied_path
                    .file_stem()
                    .map(|s| s.to_string_lossy())
                    .unwrap_or_default();
                let added_suffix =
                    if let Some(extension) = copied_path.extension().map(|e| e.to_string_lossy()) {
                        format!("{}{}.{}", stem, suffix, extension)
                    } else {
                        format!("{}{}", stem, suffix)
                    };

                current_path.join(added_suffix)
            } else {
                copied_path
            };

            if (metadata.is_file() || metadata.is_symlink())
                && (!crate::misc::exists_item(&copied_path) || is_overwrite)
            {
                if let Err(e) = std::fs::copy(file, &copied_path) {
                    crate::sys_log!("w", "Paste from clipboard failed: {}", e.kind());
                    crate::log!("Paste failed: \"{}\"", e.kind());
                }
            }

            if metadata.is_dir() {
                for entry in walkdir::WalkDir::new(file).into_iter().flatten() {
                    if entry.file_type().is_dir() {
                        continue;
                    }

                    let Ok(rel_path) = entry.path().strip_prefix(file) else {
                        continue;
                    };

                    let copied_path = copied_path.join(rel_path);

                    if !crate::misc::exists_item(&copied_path) || is_overwrite {
                        let parent = crate::misc::parent(&copied_path);

                        if !parent.exists() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                crate::sys_log!("w", "Command Paste failed: {}", e.kind());
                                crate::log!("Paste failed: \"{}\"", e.kind());

                                continue;
                            }
                        }

                        if let Err(e) = std::fs::copy(entry.path(), &copied_path) {
                            crate::sys_log!("w", "Command Paste failed: {}", e.kind());
                            crate::log!("Paste failed: \"{}\"", e.kind());
                        }
                    }
                }
            }
        }

        let body_state = self.body_state.read().unwrap();

        body_state
            .cursor
            .resize(crate::misc::child_files_len(current_path));
        crate::sys_log!("i", "Command Paste successful: {} files", files.len());
        crate::log!("{} files paste successful.", files.len());

        Ok(())
    }
}

struct AskPaste {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for AskPaste {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();

        let default = if app_state.config.get().paste.default_overwrite {
            "y"
        } else {
            ""
        };

        self.body_state
            .write()
            .unwrap()
            .input
            .enable(default, Some("Paste".into()));

        Ok(())
    }
}

struct Rename {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
    content: String,
}

impl Command for Rename {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let path = app_state.path.get();
        let body_state = self.body_state.read().unwrap();
        let cursor = &body_state.cursor;

        if let Some(under_cursor_file) = crate::misc::sorted_child_files(path).get(cursor.current())
        {
            let renamed = path.join(&self.content);

            let Ok(metadata) = under_cursor_file.symlink_metadata() else {
                crate::sys_log!("w", "Command Rename failed: target metadata cannot access");
                crate::log!("Rename failed: cannot access metadata");

                return Ok(());
            };

            if !under_cursor_file.exists() && !metadata.is_symlink() {
                crate::sys_log!("w", "Command Rename failed: target file not exists");
                crate::log!("Rename failed: \"{}\" is not exists", self.content);

                return Ok(());
            }

            if let Err(e) = std::fs::rename(under_cursor_file, &renamed) {
                crate::sys_log!("w", "Command Rename failed: {}", e.kind());
                crate::log!("Rename failed: {}", e.kind());

                return Ok(());
            }

            crate::sys_log!(
                "i",
                "Command Rename successful: \"{}\" into the \"{}\"",
                under_cursor_file.to_string_lossy(),
                renamed.to_string_lossy()
            );
            crate::log!(
                "\"{}\" renamed to \"{}\"",
                crate::misc::file_name(under_cursor_file),
                crate::misc::file_name(&renamed)
            );
        }

        Ok(())
    }
}

struct AskRename {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for AskRename {
    fn run(&self) -> Result<(), crate::Error> {
        let mut body_state = self.body_state.write().unwrap();

        body_state.selection.disable();

        if let Some(under_cursor_file) =
            crate::misc::sorted_child_files(self.app_state.read().unwrap().path.get())
                .get(body_state.cursor.current())
        {
            let name = crate::misc::file_name(under_cursor_file);

            self.body_state
                .write()
                .unwrap()
                .input
                .enable(name, Some("Rename".into()));
            crate::sys_log!("i", "Called command: Rename");
            crate::log!("Enter new name for \"{}\"", name);
        }

        Ok(())
    }
}

struct Yank {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for Yank {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let body_state = self.body_state.read().unwrap();
        let path = app_state.path.get();
        let cursor = &body_state.cursor;

        let Some(under_cursor_file) = crate::misc::sorted_child_files(path)
            .get(cursor.current())
            .cloned()
        else {
            crate::sys_log!("w", "File yank failed: invalid cursor position");
            crate::log!("Yank failed: invalid cursor position");

            return Ok(());
        };

        let is_native = app_state.config.get().native_clip;

        if is_native {
            let text = format!("file://{}", under_cursor_file.to_string_lossy());

            if let Err(e) = crate::clipboard::clip_native(&text, "text/uri-list") {
                crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                crate::log!("Yank failed: {}", e.kind());

                return Ok(());
            }
        } else {
            crate::clipboard::clip(&under_cursor_file.to_string_lossy());
        }

        crate::sys_log!(
            "i",
            "File the {} yanked",
            under_cursor_file.to_string_lossy()
        );
        crate::log!("Yanked \"{}\"", crate::misc::file_name(&under_cursor_file));

        Ok(())
    }
}

struct YankSelected {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<AppState>>,
}

impl Command for YankSelected {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let mut body_state = self.body_state.write().unwrap();
        let path = app_state.path.get();
        let selection = &mut body_state.selection;
        let is_native = app_state.config.get().native_clip;
        let selected_files = crate::misc::sorted_child_files(path)
            .into_iter()
            .enumerate()
            .filter_map(|(i, f)| {
                selection
                    .is_selected(i)
                    .then_some(f.to_string_lossy().to_string())
            })
            .map(|p| {
                if is_native {
                    format!("file://{}", p)
                } else {
                    p
                }
            })
            .collect::<Vec<_>>();

        if is_native {
            if let Err(e) =
                crate::clipboard::clip_native(&selected_files.join("\n"), "text/uri-list")
            {
                crate::sys_log!("w", "Native file yank command failed: {}", e.kind());
                crate::log!("Yank failed: {}", e.kind());

                return Ok(());
            }
        } else {
            crate::clipboard::clip(&selected_files.join("\n"));
        }

        selection.disable();
        crate::sys_log!("i", "{} files yanked", selected_files.len());
        crate::log!("Yanked {} items", selected_files.len());

        Ok(())
    }
}

struct Search {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
}

impl Command for Search {
    fn run(&self) -> Result<(), crate::Error> {
        let mut body_state = self.body_state.write().unwrap();

        body_state.selection.disable();
        body_state.grep.reset();
        body_state.input.enable("/", Some("Search".into()));

        Ok(())
    }
}

struct SearchNext {
    body_state: std::sync::Arc<std::sync::RwLock<BodyState>>,
    app_state: std::sync::Arc<std::sync::RwLock<super::app::AppState>>,
}

impl Command for SearchNext {
    fn run(&self) -> Result<(), crate::Error> {
        let app_state = self.app_state.read().unwrap();
        let path = app_state.path.get();
        let child_files = crate::misc::sorted_child_files(path);
        let body_state = self.body_state.read().unwrap();
        let cursor = &body_state.cursor;
        let current_pos = cursor.current();
        let grep = &body_state.grep;

        let first_match_pos = child_files[current_pos + 1..]
            .iter()
            .chain(child_files[..current_pos].iter())
            .position(|f| grep.is_match_found(crate::misc::file_name(f)))
            .map(|pos| pos + 1)
            .unwrap_or(0);

        cursor.shift_loop_p(first_match_pos);
        crate::log!("/{}", grep.inner);

        Ok(())
    }
}

impl Component for Body {
    fn on_init(&self) -> Result<(), crate::Error> {
        use super::app::Mode;

        {
            let mut lock = self.root_state.write().unwrap();
            let prenum = lock.key_buffer.prenum();
            let registry = &mut lock.mapping_registry;

            registry.register_key(
                Mode::Normal,
                "j".parse()?,
                MoveDown {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Visual,
                "j".parse()?,
                MoveDown {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Normal,
                "k".parse()?,
                MoveUp {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Visual,
                "k".parse()?,
                MoveUp {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Normal,
                "gg".parse()?,
                MoveTop {
                    state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "gg".parse()?,
                MoveTop {
                    state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "G".parse()?,
                MoveBottom {
                    state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "G".parse()?,
                MoveBottom {
                    state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "gj".parse()?,
                PageDown {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Visual,
                "gj".parse()?,
                PageDown {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Normal,
                "gk".parse()?,
                PageUp {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Visual,
                "gk".parse()?,
                PageUp {
                    state: self.state.clone(),
                    prenum: prenum.unwrap_or(1),
                },
            );
            registry.register_key(
                Mode::Normal,
                "h".parse()?,
                MoveParent {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "h".parse()?,
                MoveParent {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "l".parse()?,
                EnterDirOrEdit {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "l".parse()?,
                EnterDirOrEdit {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "V".parse()?,
                VisualSelect {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "V".parse()?,
                VisualSelect {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "a".parse()?,
                AskCreate {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "a".parse()?,
                AskCreate {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );

            let is_ask = self.app_state.read().unwrap().config.get().delete.ask;

            if is_ask {
                registry.register_key(
                    Mode::Normal,
                    "d".parse()?,
                    AskDelete {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
                registry.register_key(
                    Mode::Visual,
                    "d".parse()?,
                    AskDelete {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
            } else {
                registry.register_key(
                    Mode::Normal,
                    "dd".parse()?,
                    DeleteFileOrDir {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
                registry.register_key(
                    Mode::Visual,
                    "d".parse()?,
                    DeleteSelected {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
            }

            let is_force_paste = self.app_state.read().unwrap().config.get().paste.force_mode;

            if is_force_paste {
                registry.register_key(
                    Mode::Normal,
                    "p".parse()?,
                    Paste {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
                registry.register_key(
                    Mode::Visual,
                    "p".parse()?,
                    Paste {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
            } else {
                registry.register_key(
                    Mode::Normal,
                    "p".parse()?,
                    AskPaste {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
                registry.register_key(
                    Mode::Visual,
                    "p".parse()?,
                    AskPaste {
                        body_state: self.state.clone(),
                        app_state: self.app_state.clone(),
                    },
                );
            }
            registry.register_key(
                Mode::Normal,
                "r".parse()?,
                AskRename {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "r".parse()?,
                AskRename {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "yy".parse()?,
                Yank {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "y".parse()?,
                YankSelected {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "/".parse()?,
                Search {
                    body_state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "/".parse()?,
                Search {
                    body_state: self.state.clone(),
                },
            );
            registry.register_key(
                Mode::Normal,
                "n".parse()?,
                SearchNext {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
            registry.register_key(
                Mode::Visual,
                "n".parse()?,
                SearchNext {
                    body_state: self.state.clone(),
                    app_state: self.app_state.clone(),
                },
            );
        }

        self.inner.iter().try_for_each(|c| c.on_init())?;

        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        self.inner.iter().try_for_each(|c| c.on_tick())?;

        if matches!(self.app_state.read().unwrap().mode, super::app::Mode::Input) {
            return Ok(());
        }

        let (action, content) = {
            let mut lock = self.state.write().unwrap();
            let input = &mut lock.input;

            (input.drain_action(), input.drain_storage())
        };

        let Some(content) = content else {
            return Ok(());
        };

        let app_state = self.app_state.clone();
        let body_state = self.state.clone();

        tokio::task::spawn_blocking(move || {
            if let Some(action) = action {
                {
                    let mut lock = app_state.write().unwrap();
                    let proc_counter = &mut lock.process_counter;

                    proc_counter.up();
                }

                {
                    if let Err(e) = match action.as_str() {
                        "CreateFileOrDir" => CreateFileOrDir {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                            content: content.to_owned(),
                            is_file: !content.ends_with("/"),
                        }
                        .run(),
                        "DeleteFileOrDir" => DeleteFileOrDir {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                        }
                        .run(),
                        "DeleteSelected" => DeleteSelected {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                        }
                        .run(),
                        "Paste" => Paste {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                        }
                        .run(),
                        "Rename" => Rename {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                            content: content.to_owned(),
                        }
                        .run(),
                        "Search" => SearchNext {
                            body_state: body_state.clone(),
                            app_state: app_state.clone(),
                        }
                        .run(),
                        _ => Ok(()),
                    } {
                        e.handle();
                    };
                }

                {
                    let mut lock = app_state.write().unwrap();
                    let proc_counter = &mut lock.process_counter;

                    proc_counter.down();
                }
            }
        });

        Ok(())
    }
}
