use super::{ConfigModel, KeymapConfig, KeymapRegistry, theme};
use std::io;
use viks::Keymap;

pub async fn setup_local() -> io::Result<()> {
    use std::fs;

    let config_path = super::file_path();

    if !config_path.exists() {
        let parent = config_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let default_config = toml::to_string_pretty(&ConfigModel::default())
            .expect("Wrong default config: code bug");

        fs::write(config_path, default_config)?;
    }

    let theme_dir = theme::dir_path();

    if !theme_dir.exists() {
        fs::create_dir_all(&theme_dir)?;
    }

    if theme_dir.read_dir().is_ok_and(|dir| dir.count() == 0) {
        theme::download_official_theme("dark").await?;
    }

    Ok(())
}

macro_rules! nmap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register(Mode::Normal, Keymap::new($keys), $exec) }};
}

macro_rules! vmap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register(Mode::Visual, Keymap::new($keys), $exec) }};
}

macro_rules! imap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register(Mode::Input, Keymap::new($keys), $exec) }};
}

pub(super) fn init_keymaps(registry: &mut KeymapRegistry, keyconf: &Option<KeymapConfig>) {
    use crate::{proc::Command, state::Mode};

    init_builtin_keymaps(registry);

    if let Some(keyconf) = keyconf {
        if let Some(ref normal) = keyconf.normal {
            normal
                .0
                .iter()
                .filter_map(|(key, value)| Some((Keymap::new(key), Keymap::new(value).ok()?)))
                .for_each(|(key, value)| {
                    registry.register(
                        Mode::Normal,
                        key,
                        Command(move |state, _| {
                            let keymaps = &super::get().keymaps;
                            let mut cmds = keymaps
                                .eval_keys(Mode::Normal, value.as_vec().clone())
                                .into_iter();

                            while let Some(Ok((cmd, ctx))) = cmds.next() {
                                cmd.run(state.clone(), ctx);
                            }
                        }),
                    )
                });
        }

        if let Some(ref visual) = keyconf.visual {
            visual
                .0
                .iter()
                .filter_map(|(key, value)| Some((Keymap::new(key), Keymap::new(value).ok()?)))
                .for_each(|(key, value)| {
                    registry.register(
                        Mode::Visual,
                        key,
                        Command(move |state, _| {
                            let keymaps = &super::get().keymaps;
                            let mut cmds = keymaps
                                .eval_keys(Mode::Visual, value.as_vec().clone())
                                .into_iter();

                            while let Some(Ok((cmd, ctx))) = cmds.next() {
                                cmd.run(state.clone(), ctx);
                            }
                        }),
                    )
                });
        }
    }
}

fn init_builtin_keymaps(r: &mut KeymapRegistry) {
    use crate::{
        proc::{Acommand, Command, input, view},
        state::Mode,
        tui,
    };

    nmap!(r, "<ESC>", Command(|s, _| view::refresh(s)));
    nmap!(r, "ZZ", Command(|_, _| tui::close()));
    nmap!(r, "j", Command(|s, ctx| view::move_cursor(s, ctx, true)));
    nmap!(r, "k", Command(|s, ctx| view::move_cursor(s, ctx, false)));
    nmap!(r, "G", Command(|s, _| view::move_cursor_too(s, true)));
    nmap!(r, "gg", Command(|s, _| view::move_cursor_too(s, false)));
    nmap!(r, "gj", Command(|s, ctx| view::move_page(s, ctx, true)));
    nmap!(r, "gk", Command(|s, ctx| view::move_page(s, ctx, false)));
    nmap!(r, "h", Command(|s, _| view::move_parent(s)));
    nmap!(r, "l", Command(|s, _| view::attach_child(s)));
    nmap!(r, "V", Command(|s, _| view::toggle_vis(s)));
    nmap!(r, "a", Command(|s, _| input::ask_create(s)));
    nmap!(r, "dd", Command(|s, _| input::ask_delete(s)));
    nmap!(r, "yy", Command(|_s, _| todo!()));
    nmap!(r, "r", Command(|s, _| input::ask_rename(s)));

    vmap!(r, "<ESC>", Command(|s, _| view::refresh(s)));
    vmap!(r, "ZZ", Command(|_, _| tui::close()));
    vmap!(r, "j", Command(|s, ctx| view::move_cursor(s, ctx, true)));
    vmap!(r, "k", Command(|s, ctx| view::move_cursor(s, ctx, false)));
    vmap!(r, "G", Command(|s, _| view::move_cursor_too(s, true)));
    vmap!(r, "gg", Command(|s, _| view::move_cursor_too(s, false)));
    vmap!(r, "gj", Command(|s, ctx| view::move_page(s, ctx, true)));
    vmap!(r, "gk", Command(|s, ctx| view::move_page(s, ctx, false)));
    vmap!(r, "h", Command(|s, _| view::move_parent(s)));
    vmap!(r, "l", Command(|s, _| view::attach_child(s)));
    vmap!(r, "V", Command(|s, _| view::toggle_vis(s)));
    vmap!(r, "a", Command(|s, _| input::ask_create(s)));
    vmap!(r, "d", Command(|s, _| input::ask_delete_selects(s)));
    vmap!(r, "y", Command(|_s, _| todo!()));
    vmap!(r, "r", Command(|s, _| input::ask_rename(s)));

    imap!(r, "<ESC>", Command(|s, _| input::restore(s)));
    imap!(r, "<ENTER>", Acommand(|s, _| input::complete_input(s)));
    imap!(r, "<BS>", Command(|s, _| s.input.input.pop()));
    imap!(r, "<DEL>", Command(|s, _| s.input.input.pop_front()));

    imap!(r, "<c-h>", Command(|s, _| s.input.input.shift_back()));
    imap!(r, "<c-l>", Command(|s, _| s.input.input.shift()));

    imap!(r, "<SPACE>", Command(|s, _| s.input.input.put(' ')));
    imap!(r, "<LT>", Command(|s, _| s.input.input.put('<')));

    for i_key in ('!'..='~').filter(|c| *c != '<') {
        imap!(
            r,
            &i_key.to_string(),
            Command(move |s, _| s.input.input.put(i_key))
        );
    }
}
