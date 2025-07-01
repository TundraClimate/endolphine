use crate::{proc::Runnable, state::Mode, theme::Theme};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
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

    pub fn split_to_maps(&self, mode: Mode, keys: Vec<Key>) -> Vec<Result<Vec<Key>, Vec<Key>>> {
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
}

#[derive(Deserialize, Serialize)]
struct ConfigModel {
    editor: Vec<String>,
    theme: String,
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

        init_keymaps(&mut keymaps);

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
        }
    }
}

fn init_keymaps(registry: &mut KeymapRegistry) {
    use crate::{proc::Command, state::Mode, tui};

    registry.register(
        Mode::Normal,
        Keymap::new("ZZ"),
        Command(|_, _| tui::close()),
    );
}
