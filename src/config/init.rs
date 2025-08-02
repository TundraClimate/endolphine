use super::{ConfigModel, KeymapConfig, KeymapRegistry, theme};
use crate::state::Mode;
use std::io;
use viks::Keymap;

pub async fn setup_local() -> io::Result<()> {
    use std::{fs, path::Path};

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

    let tmp_dir = Path::new("/tmp").join("endolphine");

    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir)?;
    }

    Ok(())
}

macro_rules! nmap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register_raw(Mode::Normal, Keymap::new($keys), $exec) }};
}

macro_rules! vmap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register_raw(Mode::Visual, Keymap::new($keys), $exec) }};
}

macro_rules! imap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register_raw(Mode::Input, Keymap::new($keys), $exec) }};
}

macro_rules! smap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register_raw(Mode::Search, Keymap::new($keys), $exec) }};
}

macro_rules! mmap {
    ($registry:expr, $keys:expr, $exec:expr $(,)?) => {{ $registry.register_raw(Mode::Menu, Keymap::new($keys), $exec) }};
}

fn register_remap(registry: &mut KeymapRegistry, mode: Mode, maps: Vec<(Keymap, Keymap)>) {
    use crate::proc::Command;

    maps.into_iter().for_each(|(from, expand)| {
        registry.register(
            mode,
            from,
            Command(move |state, _| {
                let keymaps = &super::get().keymaps;
                let mut cmds = keymaps.eval_keys(mode, expand.as_vec().clone()).into_iter();

                while let Some(Ok((cmd, ctx))) = cmds.next() {
                    cmd.run(state.clone(), ctx);
                }
            }),
        )
    })
}

pub(super) fn init_keymaps(registry: &mut KeymapRegistry, keyconf: &Option<KeymapConfig>) {
    log::info!("Initialize builtin keymaps");

    init_builtin_keymaps(registry);

    log::info!("Initialize user-defined keymaps");

    if let Some(keyconf) = keyconf {
        if let Some(ref normal) = keyconf.normal {
            register_remap(registry, Mode::Normal, normal.collect_maps());
        }

        if let Some(ref visual) = keyconf.visual {
            register_remap(registry, Mode::Visual, visual.collect_maps());
        }

        if let Some(ref menu) = keyconf.menu {
            register_remap(registry, Mode::Menu, menu.collect_maps());
        }
    }
}

fn init_builtin_keymaps(r: &mut KeymapRegistry) {
    use crate::{
        proc::{
            Acommand, Command,
            input::{self, search},
            menu, view, yank,
        },
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
    nmap!(r, "yy", Command(|s, _| yank::yank(s)));
    nmap!(r, "r", Command(|s, _| input::ask_rename(s)));
    nmap!(r, "p", Command(|s, _| input::ask_paste(s)));
    nmap!(r, "/", Command(|s, _| search::start_search(s)));
    nmap!(r, "n", Command(|s, _| search::search_next(s)));
    nmap!(r, "M", Command(|s, _| menu::toggle_menu_open(s)));
    nmap!(r, "m", Command(|s, _| menu::toggle_menu(s)));

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
    vmap!(r, "y", Command(|s, _| yank::yank_selects(s)));
    vmap!(r, "r", Command(|s, _| input::ask_rename(s)));
    vmap!(r, "p", Command(|s, _| input::ask_paste(s)));
    vmap!(r, "/", Command(|s, _| search::start_search(s)));
    vmap!(r, "n", Command(|s, _| search::search_next(s)));
    vmap!(r, "M", Command(|s, _| menu::toggle_menu_open(s)));
    vmap!(r, "m", Command(|s, _| menu::toggle_menu(s)));

    imap!(r, "<ESC>", Command(|s, _| input::restore(s)));
    imap!(r, "<ENTER>", Acommand(|s, _| input::complete_input(s)));
    imap!(r, "<BS>", Command(|s, _| input::pop(s)));
    imap!(r, "<DEL>", Command(|s, _| input::pop_front(s)));

    imap!(r, "<c-h>", Command(|s, _| s.input.input.shift_back()));
    imap!(r, "<c-l>", Command(|s, _| s.input.input.shift()));

    imap!(r, "<SPACE>", Command(|s, _| input::put(s, ' ')));
    imap!(r, "<LT>", Command(|s, _| input::put(s, '<')));

    for i_key in ('!'..='~').filter(|c| *c != '<') {
        imap!(
            r,
            &i_key.to_string(),
            Command(move |s, _| input::put(s, i_key))
        );
    }

    imap!(r, "y", Command(|s, _| input::answer_or_put(s, 'y')));
    imap!(r, "n", Command(|s, _| input::answer_or_put(s, 'n')));
    imap!(r, "Y", Command(|s, _| input::answer_or_put(s, 'Y')));
    imap!(r, "N", Command(|s, _| input::answer_or_put(s, 'N')));

    smap!(r, "<ESC>", Command(|s, _| input::restore(s)));
    smap!(r, "<ENTER>", Command(|s, _| input::complete_input(s)));
    smap!(r, "<BS>", Command(|s, _| search::pop(s)));
    smap!(r, "<DEL>", Command(|s, _| search::pop_front(s)));

    smap!(r, "<c-h>", Command(|s, _| s.input.input.shift_back()));
    smap!(r, "<c-l>", Command(|s, _| s.input.input.shift()));

    smap!(r, "<SPACE>", Command(|s, _| search::put(s, ' ')));
    smap!(r, "<LT>", Command(|s, _| search::put(s, '<')));

    for i_key in ('!'..='~').filter(|c| *c != '<') {
        smap!(
            r,
            &i_key.to_string(),
            Command(move |s, _| search::put(s, i_key))
        );
    }

    mmap!(r, "ZZ", Command(|_, _| tui::close()));
    mmap!(r, "M", Command(|s, _| menu::toggle_menu_open(s)));
    mmap!(r, "m", Command(|s, _| menu::toggle_menu(s)));
    mmap!(r, "j", Command(|s, ctx| menu::move_cursor(s, ctx, true)));
    mmap!(r, "k", Command(|s, ctx| menu::move_cursor(s, ctx, false)));
    mmap!(r, "G", Command(|s, _| menu::move_cursor_too(s, true)));
    mmap!(r, "gg", Command(|s, _| menu::move_cursor_too(s, false)));
    mmap!(r, "l", Command(|s, _| menu::enter(s)));
}
