use crate::{
    builtin, command, config, global,
    key::{Key, Keymap},
    menu::MenuElement,
    theme::{self, Scheme, Theme},
};
use std::path::{Path, PathBuf};

pub fn file_path() -> Option<PathBuf> {
    option_env!("HOME").map(|home| {
        Path::new(home)
            .join(".config")
            .join("endolphine")
            .join("config.toml")
    })
}

pub async fn edit() {
    let editor = option_env!("EDITOR").unwrap_or("vi");

    let Some(config_path) = config::file_path() else {
        panic!("Open error: Config not initialized");
    };

    tokio::process::Command::new(editor)
        .arg(config_path)
        .status()
        .await
        .ok();
}

pub fn check() -> Result<(), (toml::de::Error, String)> {
    if let Some(Err(e)) = try_load() {
        let config = config::file_path().and_then(|p| std::fs::read_to_string(p).ok());
        if let (Some(config), Some(span)) = (config, e.span()) {
            let lines = config
                .char_indices()
                .collect::<Vec<_>>()
                .split(|(_, c)| *c == '\n')
                .enumerate()
                .filter_map(|(row, line)| {
                    line.iter()
                        .any(|(i, _)| span.contains(i))
                        .then_some(format!(
                            "{}: {}",
                            row + 1,
                            line.iter().map(|(_, c)| *c).collect::<String>()
                        ))
                })
                .collect::<Vec<_>>();

            return Err((e, lines.join("\n")));
        }
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub editor: Vec<String>,
    theme: Theme,
    user_theme_path: Option<PathBuf>,
    pub sort_by_priority: [u8; 4],
    pub native_clip: bool,
    pub delete: DeleteConfig,
    pub paste: PasteConfig,
    pub menu: MenuConfig,
    pub keymap: Option<KeymapConfig>,
    pub open: Option<OpenConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: vec!["vim"].into_iter().map(ToString::to_string).collect(),
            sort_by_priority: [0, 1, 2, 3],
            user_theme_path: None,
            native_clip: true,
            delete: DeleteConfig {
                ask: true,
                for_tmp: true,
                yank: true,
            },
            paste: PasteConfig {
                copied_suffix: String::from("_Copy"),
                force_mode: true,
                default_overwrite: true,
            },
            menu: MenuConfig::default(),
            theme: Theme::Dark,
            keymap: None,
            open: None,
        }
    }
}

impl Config {
    fn load_user_theme(&self) -> Option<Scheme> {
        if let Some(ref path) = self.user_theme_path {
            if !path.exists() {
                crate::log!(
                    "Couldn't load the user-defined theme: \"{}\" is not exists",
                    path.to_string_lossy()
                );
                return None;
            }

            let Ok(read_content) = std::fs::read_to_string(path) else {
                crate::log!(
                    "Couldn't load the user-defined theme: unable to read the \"{}\"",
                    path.to_string_lossy()
                );
                return None;
            };

            let Ok(parsed): Result<theme::SchemeWrap, toml::de::Error> =
                toml::from_str(&read_content)
            else {
                crate::log!(
                    "Couldn't load the user-defined theme: invalid syntax in the content of \"{}\"",
                    path.to_string_lossy()
                );
                return None;
            };

            return Some(parsed.into());
        }
        None
    }

    pub fn scheme(&self) -> Scheme {
        if let Some(usr_theme) = self.load_user_theme() {
            return usr_theme;
        }

        match self.theme {
            Theme::Dark => builtin::Dark.into(),
            Theme::DarkNoBg => builtin::DarkNoBg.into(),
            Theme::Light => builtin::Light.into(),
            Theme::LightNoBg => builtin::LightNoBg.into(),
            Theme::Mars => builtin::Mars.into(),
            Theme::Neon => builtin::Neon.into(),
            Theme::Ice => builtin::Ice.into(),
            Theme::Nept => builtin::Nept.into(),
            Theme::Volcano => builtin::Volcano.into(),
            Theme::Mossy => builtin::Mossy.into(),
            Theme::Monochrome => builtin::Monochrome.into(),
            Theme::Holiday => builtin::Holiday.into(),
            Theme::Bloom => builtin::Bloom.into(),
            Theme::Collapse => builtin::Collapse.into(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DeleteConfig {
    pub ask: bool,
    pub yank: bool,
    pub for_tmp: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PasteConfig {
    copied_suffix: String,
    pub force_mode: bool,
    pub default_overwrite: bool,
}

impl PasteConfig {
    pub fn similar_file_suffix(&self) -> String {
        let mut suf = self.copied_suffix.trim().to_string();
        suf.retain(|c| !c.is_whitespace());
        suf
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MenuConfig {
    pub items: Vec<MenuElement>,
}

impl Default for MenuConfig {
    fn default() -> Self {
        let home_path = option_env!("HOME").unwrap_or("/root");
        let dls_path = format!("{}/Downloads", home_path);
        let desktop_path = format!("{}/Desktop", home_path);
        MenuConfig {
            items: vec![
                MenuElement::new("Home", home_path),
                MenuElement::new("Downloads", dls_path),
                MenuElement::new("Desktop", desktop_path),
            ],
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeymapNormal(std::collections::BTreeMap<String, String>);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeymapVisual(std::collections::BTreeMap<String, String>);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeymapInput(std::collections::BTreeMap<String, String>);

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeymapConfig {
    normal: Option<KeymapNormal>,
    visual: Option<KeymapVisual>,
    input: Option<KeymapInput>,
}

impl KeymapConfig {
    pub fn normal_key_map(&self) -> Option<Vec<(Keymap, command::Remapping)>> {
        self.normal.as_ref().and_then(|normal| {
            normal
                .0
                .keys()
                .zip(normal.0.values())
                .try_fold(
                    Vec::<(Keymap, command::Remapping)>::new(),
                    |mut acc, (key, val)| {
                        acc.push((
                            key.as_str().parse()?,
                            command::Remapping(val.as_str().parse()?),
                        ));
                        Ok::<_, crate::Error>(acc)
                    },
                )
                .ok()
        })
    }

    pub fn visual_key_map(&self) -> Option<Vec<(Keymap, command::Remapping)>> {
        self.visual.as_ref().and_then(|visual| {
            visual
                .0
                .keys()
                .zip(visual.0.values())
                .try_fold(
                    Vec::<(Keymap, command::Remapping)>::new(),
                    |mut acc, (key, val)| {
                        acc.push((
                            key.as_str().parse()?,
                            command::Remapping(val.as_str().parse()?),
                        ));
                        Ok::<_, crate::Error>(acc)
                    },
                )
                .ok()
        })
    }

    pub fn input_key_map(&self) -> Option<Vec<(Keymap, command::Remapping)>> {
        self.input.as_ref().and_then(|input| {
            input
                .0
                .keys()
                .zip(input.0.values())
                .try_fold(
                    Vec::<(Keymap, command::Remapping)>::new(),
                    |mut acc, (key, val)| {
                        acc.push((
                            key.as_str().parse()?,
                            command::Remapping(val.as_str().parse()?),
                        ));
                        Ok::<_, crate::Error>(acc)
                    },
                )
                .ok()
        })
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct OpenOpts {
    pub cmd: Vec<String>,
    pub in_term: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OpenConfig(std::collections::BTreeMap<String, OpenOpts>);

impl OpenConfig {
    pub fn corresponding_with(&self, extension: &str) -> Option<OpenOpts> {
        self.0.get(extension).cloned()
    }
}

global! {
    static CONFIG: Config = file_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_else(|| {
            crate::sys_log!("w", "load config.toml failed, use the default config");
            Config::default()
        });
}

fn try_load() -> Option<Result<Config, toml::de::Error>> {
    file_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|c| toml::from_str(&c))
}

pub fn load() -> &'static Config {
    &CONFIG
}

global! {
    static THEME: Scheme = CONFIG.scheme();
}

pub fn theme() -> &'static Scheme {
    &THEME
}

global! {
    static KEYMAP_REGISTRY: std::sync::RwLock<std::collections::HashMap<(u8, String), Box<dyn command::Command>>> =
        std::sync::RwLock::new(std::collections::HashMap::new());
}

pub fn register_key<C: command::Command + 'static>(
    mode: crate::app::AppMode,
    keymap: Keymap,
    cmd: C,
) {
    let mut lock = KEYMAP_REGISTRY.write().unwrap();
    lock.insert((mode as u8, keymap.to_string()), Box::new(cmd));
}

pub fn has_similar_map(buf: &[Key], mode: crate::app::AppMode) -> bool {
    let lock = KEYMAP_REGISTRY.read().unwrap();

    if buf.is_empty() {
        return false;
    }

    if buf.iter().all(Key::is_digit) {
        return true;
    }

    let buf = buf.iter().skip_while(|k| k.is_digit()).collect::<Vec<_>>();

    let mode = mode as u8;

    lock.keys().any(|(rmode, keymap)| {
        buf.len() <= keymap.len()
            && mode == *rmode
            && buf.iter().enumerate().all(|(i, k)| {
                keymap
                    .as_str()
                    .parse::<Keymap>()
                    .is_ok_and(|key| key.as_vec().get(i) == Some(k))
            })
    })
}

pub fn has_map(buf: &[Key], mode: crate::app::AppMode) -> bool {
    let lock = KEYMAP_REGISTRY.read().unwrap();

    if buf.is_empty() || buf.iter().all(Key::is_digit) {
        return false;
    }

    let buf = buf.iter().skip_while(|k| k.is_digit()).collect::<Vec<_>>();

    let mode = mode as u8;

    lock.keys().any(|(rmode, keymap)| {
        buf.len() == keymap.len()
            && mode == *rmode
            && buf.iter().enumerate().all(|(i, k)| {
                keymap
                    .as_str()
                    .parse::<Keymap>()
                    .is_ok_and(|key| key.as_vec().get(i) == Some(k))
            })
    })
}

pub fn eval_input_keymap(keymap: &[Key]) -> Option<Result<(), crate::Error>> {
    let lock = KEYMAP_REGISTRY.read().unwrap();
    let mode = crate::app::AppMode::Input as u8;
    lock.get(&(mode, Keymap::new(keymap).to_string()))
        .map(|cmd| cmd.run())
}

pub fn eval_keymap(mode: crate::app::AppMode, keymap: &[Key]) -> Option<Result<(), crate::Error>> {
    let lock = KEYMAP_REGISTRY.read().unwrap();
    let keymap = keymap
        .iter()
        .skip_while(|k| k.is_digit())
        .cloned()
        .collect::<Vec<Key>>();

    lock.get(&(mode as u8, Keymap::new(keymap.as_slice()).to_string()))
        .map(|cmd| cmd.run())
}
