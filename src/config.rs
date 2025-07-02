use crate::{
    proc::{CommandContext, Runnable},
    state::Mode,
    theme::Theme,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    io,
    path::{Path, PathBuf},
};
use viks::{Key, Keymap};

pub fn file_path() -> PathBuf {
    let Some(home) = option_env!("HOME") else {
        panic!("Couldn't read the $HOME");
    };

    Path::new(home)
        .join(".config")
        .join("endolphine")
        .join("config.toml")
}

pub async fn setup_local() -> io::Result<()> {
    use crate::theme;
    use std::fs;

    let config_path = file_path();

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

pub struct KeymapRegistry {
    map: HashMap<(Mode, String), Box<dyn Runnable>>,
}

impl KeymapRegistry {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    fn register<R: Runnable + 'static>(&mut self, mode: Mode, map: viks::Result<Keymap>, cmd: R) {
        self.map.insert(
            (
                mode,
                map.expect("Invalid mapping found: KeymapRegistry")
                    .to_string(),
            ),
            Box::new(cmd),
        );
    }

    fn split_to_maps(&self, mode: Mode, keys: Vec<Key>) -> Vec<Result<Vec<Key>, Vec<Key>>> {
        let maps = &self.map;
        let mut res = vec![];
        let mut prenum_buf = vec![];
        let mut buf = vec![];

        for key in keys.into_iter() {
            let as_str = key.to_string();
            let mut chars = as_str.chars();

            if buf.is_empty() && chars.all(char::is_numeric) {
                prenum_buf.push(key);

                continue;
            }

            buf.push(key);

            if maps.contains_key(&(mode, Keymap::from(buf.clone()).to_string())) {
                let mut keymap = vec![];

                keymap.append(&mut prenum_buf);
                keymap.append(&mut buf);

                res.push(Ok(keymap));
            }
        }

        if !prenum_buf.is_empty() || !buf.is_empty() {
            let mut keymap = vec![];

            keymap.append(&mut prenum_buf);
            keymap.append(&mut buf);

            res.push(Err(keymap));
        }

        res
    }

    pub fn get(&self, mode: Mode, keys: Keymap) -> Option<&dyn Runnable> {
        self.map.get(&(mode, keys.to_string())).map(|cmd| &**cmd)
    }

    pub fn has_similar_map(&self, mode: Mode, keys: Keymap) -> bool {
        self.map
            .keys()
            .filter_map(|(m, map)| (mode == *m).then_some(map))
            .filter_map(|map| Keymap::new(map).ok())
            .filter(|map| map.as_vec().len() >= keys.as_vec().len())
            .any(|map| {
                let keys = keys.as_vec();

                map.as_vec()[..keys.len()] == keys[..]
            })
    }

    #[allow(clippy::type_complexity)]
    pub fn eval_keys(
        &self,
        mode: Mode,
        keys: Vec<Key>,
    ) -> Vec<Result<(&dyn Runnable, CommandContext), Vec<Key>>> {
        let parsed = self.split_to_maps(mode, keys);

        parsed
            .into_iter()
            .map(|map| match map {
                Ok(keys) => {
                    let prenum = &keys
                        .iter()
                        .take_while(|key| key.to_string().chars().all(char::is_numeric))
                        .map(ToString::to_string)
                        .collect::<String>()
                        .parse::<usize>()
                        .ok();
                    let keys = keys
                        .into_iter()
                        .skip_while(|key| key.to_string().chars().all(char::is_numeric))
                        .collect::<Vec<_>>();

                    let item = (
                        self.get(mode, Keymap::from(keys))
                            .expect("Incorrect code found"),
                        CommandContext::new(*prenum),
                    );

                    Ok(item)
                }
                Err(keys) => Err(keys),
            })
            .collect::<Vec<_>>()
    }
}

#[derive(Deserialize, Serialize)]
struct KeymapConfig {
    normal: Option<NormalMaps>,
    visual: Option<VisualMaps>,
}

#[derive(Deserialize, Serialize)]
struct NormalMaps(BTreeMap<String, String>);

#[derive(Deserialize, Serialize)]
struct VisualMaps(BTreeMap<String, String>);

#[derive(Deserialize, Serialize)]
struct ConfigModel {
    editor: Vec<String>,
    theme: String,
    keymap: Option<KeymapConfig>,
}

pub struct Config {
    pub editor: Vec<String>,
    pub theme: Theme,
    pub keymaps: KeymapRegistry,
}

pub fn parse_check(s: &str) -> Result<(), toml::de::Error> {
    toml::from_str::<ConfigModel>(s).map(|_| ())
}

pub fn get() -> &'static Config {
    use crate::theme;
    use std::{fs, sync::LazyLock};

    static CONFIG: LazyLock<Config> = LazyLock::new(|| {
        let model = fs::read_to_string(file_path())
            .ok()
            .and_then(|config| toml::from_str::<ConfigModel>(&config).ok())
            .unwrap_or_default();
        let theme_path = theme::dir_path().join(format!("{}.toml", model.theme));
        let Some(theme) = fs::read_to_string(theme_path)
            .ok()
            .and_then(|theme| toml::from_str::<Theme>(&theme).ok())
        else {
            panic!("Failed to load theme file");
        };

        let mut keymaps = KeymapRegistry::new();

        init_keymaps(&mut keymaps, &model.keymap);

        Config {
            editor: model.editor,
            theme,
            keymaps,
        }
    });

    &CONFIG
}

impl Default for ConfigModel {
    fn default() -> Self {
        Self {
            editor: vec!["vim".to_string()],
            theme: "dark".to_string(),
            keymap: None,
        }
    }
}

fn init_keymaps(registry: &mut KeymapRegistry, keyconf: &Option<KeymapConfig>) {
    use crate::{proc::Command, state::Mode, tui};

    registry.register(
        Mode::Normal,
        Keymap::new("ZZ"),
        Command(|_, _| tui::close()),
    );

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
                            let keymaps = &get().keymaps;
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
                            let keymaps = &get().keymaps;
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
