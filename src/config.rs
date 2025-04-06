use crate::{
    builtin, command, config, global,
    key::Keymap,
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
    pub key: KeyConfig,
    pub opener: OpenConfig,
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MenuConfig {
    pub items: Vec<MenuElement>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeyConfig {
    pub exit_app: Keymap,
    pub reset_view: Keymap,
    pub move_up: Keymap,
    pub move_up_ten: Keymap,
    pub move_down: Keymap,
    pub move_down_ten: Keymap,
    pub move_parent: Keymap,
    pub enter_dir_or_edit: Keymap,
    pub visual_select: Keymap,
    pub menu_toggle: Keymap,
    pub menu_move: Keymap,
    pub create_new: Keymap,
    pub delete: Keymap,
    pub rename: Keymap,
    pub yank: Keymap,
    pub paste: Keymap,
    pub search: Keymap,
    pub search_next: Keymap,
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct OpenOpts {
    pub cmd: Vec<String>,
    pub in_term: Option<bool>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct OpenConfig(Option<std::collections::BTreeMap<String, OpenOpts>>);

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
            key: KeyConfig::default(),
            opener: OpenConfig(None),
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

impl PasteConfig {
    pub fn similar_file_suffix(&self) -> String {
        let mut suf = self.copied_suffix.trim().to_string();
        suf.retain(|c| !c.is_whitespace());
        suf
    }
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

impl KeyConfig {
    pub fn registerd() -> Vec<(Box<dyn command::Command>, Keymap)> {
        let delete_cmd: (Box<dyn command::Command>, Keymap) = if CONFIG.delete.ask {
            (Box::new(command::AskDelete), CONFIG.key.delete.clone())
        } else {
            let map = format!("{0}{0}", CONFIG.key.delete)
                .parse::<Keymap>()
                .unwrap();
            (
                Box::new(command::DeleteFileOrDir {
                    use_tmp: CONFIG.delete.for_tmp,
                    yank_and_native: (CONFIG.delete.yank, CONFIG.native_clip),
                }),
                map,
            )
        };

        vec![
            (Box::new(command::ExitApp), CONFIG.key.exit_app.clone()),
            (Box::new(command::ResetView), CONFIG.key.reset_view.clone()),
            (Box::new(command::Move(-1)), CONFIG.key.move_up.clone()),
            (Box::new(command::Move(-10)), CONFIG.key.move_up_ten.clone()),
            (Box::new(command::Move(1)), CONFIG.key.move_down.clone()),
            (
                Box::new(command::Move(10)),
                CONFIG.key.move_down_ten.clone(),
            ),
            (
                Box::new(command::MoveParent),
                CONFIG.key.move_parent.clone(),
            ),
            (
                Box::new(command::EnterDirOrEdit),
                CONFIG.key.enter_dir_or_edit.clone(),
            ),
            (
                Box::new(command::VisualSelect),
                CONFIG.key.visual_select.clone(),
            ),
            (
                Box::new(command::MenuToggle),
                CONFIG.key.menu_toggle.clone(),
            ),
            (Box::new(command::MenuMove), CONFIG.key.menu_move.clone()),
            (Box::new(command::AskCreate), CONFIG.key.create_new.clone()),
            delete_cmd,
            (Box::new(command::AskRename), CONFIG.key.rename.clone()),
            (
                Box::new(command::Yank {
                    native: config::load().native_clip,
                }),
                CONFIG.key.yank.clone(),
            ),
            (Box::new(command::AskPaste), CONFIG.key.paste.clone()),
            (Box::new(command::Search), CONFIG.key.search.clone()),
            (
                Box::new(command::SearchNext),
                CONFIG.key.search_next.clone(),
            ),
        ]
    }
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            exit_app: "Q".into(),
            reset_view: "<ESC>".into(),
            move_up: "k".into(),
            move_up_ten: "K".into(),
            move_down: "j".into(),
            move_down_ten: "J".into(),
            move_parent: "h".into(),
            enter_dir_or_edit: "l".into(),
            visual_select: "V".into(),
            menu_toggle: "M".into(),
            menu_move: "m".into(),
            create_new: "a".into(),
            delete: "d".into(),
            rename: "r".into(),
            yank: "y".into(),
            paste: "p".into(),
            search: "/".into(),
            search_next: "n".into(),
        }
    }
}

impl OpenConfig {
    pub fn corresponding_with(&self, extension: &str) -> Option<OpenOpts> {
        self.0.as_ref()?.get(extension).cloned()
    }
}

global! {
    static CONFIG: Config = file_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default();
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
