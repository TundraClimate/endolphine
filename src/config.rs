use crate::{
    config,
    error::*,
    global,
    menu::MenuElement,
    theme::{self, Scheme, Theme},
};
use std::path::{Path, PathBuf};

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

pub fn file_path() -> Option<PathBuf> {
    option_env!("HOME").map(|home| {
        Path::new(home)
            .join(".config")
            .join("endolphine")
            .join("config.toml")
    })
}

pub async fn edit() -> Result<(), crate::app::Error> {
    let editor = option_env!("EDITOR").unwrap_or("vi");

    let Some(config_path) = config::file_path() else {
        panic!("Open error: Config not initialized");
    };

    tokio::process::Command::new(editor)
        .arg(config_path)
        .status()
        .await
        .map_err(|e| crate::app::Error::CommandRun(editor.to_string(), e.kind().to_string()))?;

    Ok(())
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
    editor: Vec<String>,
    theme: Theme,
    user_theme_path: Option<PathBuf>,
    pub sort_by_priority: [u8; 4],
    pub rm: RmConfig,
    pub paste: PasteConfig,
    pub menu: MenuConfig,
    pub key: KeyConfig,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RmConfig {
    pub no_enter: bool,
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
    pub exit_app: char,
    pub move_up: char,
    pub move_up_ten: char,
    pub move_down: char,
    pub move_down_ten: char,
    pub move_parent: char,
    pub enter_dir_or_edit: char,
    pub visual_select: char,
    pub menu_toggle: char,
    pub menu_move: char,
    pub create_new: char,
    pub delete: char,
    pub rename: char,
    pub yank: char,
    pub paste: char,
    pub search: char,
    pub search_next: char,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: vec!["vim"].into_iter().map(ToString::to_string).collect(),
            sort_by_priority: [0, 1, 2, 3],
            user_theme_path: None,
            rm: RmConfig {
                no_enter: true,
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
        }
    }
}

impl Config {
    pub fn editor_command(&self) -> Option<std::process::Command> {
        let (cmd, args) = self.editor.split_first()?;
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        Some(command)
    }

    fn load_user_theme(&self) -> Option<Scheme> {
        if let Some(ref path) = self.user_theme_path {
            if !path.exists() {
                crate::log!(format!(
                    "Couldn't load the user-defined theme: \"{}\" is not exists",
                    path.to_string_lossy()
                ));
                return None;
            }

            let Ok(read_content) = std::fs::read_to_string(path) else {
                crate::log!(format!(
                    "Couldn't load the user-defined theme: unable to read the \"{}\"",
                    path.to_string_lossy()
                ));
                return None;
            };

            let Ok(parsed): Result<theme::SchemeWrap, toml::de::Error> =
                toml::from_str(&read_content)
            else {
                crate::log!(format!(
                    "Couldn't load the user-defined theme: invalid syntax in the content of \"{}\"",
                    path.to_string_lossy()
                ));
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
            Theme::Dark => theme::dark::SCHEME.into(),
            Theme::DarkNoBg => theme::dark_no_bg::SCHEME.into(),
            Theme::Light => theme::light::SCHEME.into(),
            Theme::LightNoBg => theme::light_no_bg::SCHEME.into(),
            Theme::Mars => theme::mars::SCHEME.into(),
            Theme::Neon => theme::neon::SCHEME.into(),
            Theme::Ice => theme::ice::SCHEME.into(),
            Theme::Nept => theme::nept::SCHEME.into(),
            Theme::Volcano => theme::volcano::SCHEME.into(),
            Theme::Mossy => theme::mossy::SCHEME.into(),
            Theme::Monochrome => theme::monochrome::SCHEME.into(),
            Theme::Holiday => theme::holiday::SCHEME.into(),
            Theme::Bloom => theme::bloom::SCHEME.into(),
            Theme::Collapse => theme::collapse::SCHEME.into(),
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

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            exit_app: 'Q',
            move_up: 'k',
            move_up_ten: 'K',
            move_down: 'j',
            move_down_ten: 'J',
            move_parent: 'h',
            enter_dir_or_edit: 'l',
            visual_select: 'V',
            menu_toggle: 'M',
            menu_move: 'm',
            create_new: 'a',
            delete: 'd',
            rename: 'r',
            yank: 'y',
            paste: 'p',
            search: '/',
            search_next: 'n',
        }
    }
}
