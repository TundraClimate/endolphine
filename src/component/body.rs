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
pub struct BodyState {
    cursor: crate::cursor::Cursor,
    selection: Selection,
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
        self.body_state.write().unwrap().selection.disable();
        let mut lock = self.app_state.write().unwrap();

        lock.input.enable("", Some("CreateFileOrDir".into()));
        lock.mode = super::app::Mode::Input;

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
        let body_state = self.body_state.read().unwrap();
        let under_cursor_file =
            crate::misc::sorted_child_files(self.app_state.read().unwrap().path.get())
                .get(body_state.cursor.current())
                .cloned();
        let mut app_state = self.app_state.write().unwrap();
        let input = &mut app_state.input;
        let selection = &body_state.selection;

        if selection.is_active() {
            input.enable("", Some("DeleteSelected".into()));
            crate::sys_log!("i", "Called command: DeleteSelected");

            let selected_len = selection
                .inner
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
        }

        Ok(())
    }

    fn on_tick(&self) -> Result<(), crate::Error> {
        if matches!(self.app_state.read().unwrap().mode, super::app::Mode::Input) {
            return Ok(());
        }

        let (action, content) = {
            let mut lock = self.app_state.write().unwrap();
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
