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

pub(super) fn init_keymaps(registry: &mut KeymapRegistry, keyconf: &Option<KeymapConfig>) {
    use crate::{proc::Command, state::Mode, tui};

    nmap!(registry, "ZZ", Command(|_, _| tui::close()));

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
