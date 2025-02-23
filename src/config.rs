use crate::menu::MenuElement;
use std::path::{Path, PathBuf};

pub fn file_path() -> Option<PathBuf> {
    option_env!("HOME").map(|home| {
        Path::new(home)
            .join(".config")
            .join("endolphine")
            .join("config.toml")
    })
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    editor: Vec<String>,
    pub sort_by_priority: [u8; 4],
    pub rm: RmConfig,
    pub paste: PasteConfig,
    pub menu: MenuConfig,
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

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: vec!["vim"].into_iter().map(ToString::to_string).collect(),
            sort_by_priority: [0, 1, 2, 3],
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
        }
    }
}

impl Config {
    pub fn try_load() -> Option<Result<Config, toml::de::Error>> {
        file_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|c| toml::from_str(&c))
    }

    pub fn load() -> Config {
        file_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn editor_command(&self) -> Option<std::process::Command> {
        let (cmd, args) = self.editor.split_first()?;
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        Some(command)
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
