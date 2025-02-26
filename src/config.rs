use crate::{
    config,
    error::*,
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

pub async fn edit_and_check() -> EpResult<()> {
    let editor = option_env!("EDITOR").unwrap_or("vi");

    let Some(config_path) = config::file_path() else {
        panic!("Open error: Config not initialized");
    };

    tokio::process::Command::new(editor)
        .arg(config_path)
        .status()
        .await
        .map_err(|e| EpError::CommandExecute(editor.to_string(), e.kind().to_string()))?;

    if let Some(Err(e)) = config::Config::try_load() {
        let config = config::file_path().and_then(|p| std::fs::read_to_string(p).ok());
        let position_lines = if let (Some(config), Some(span)) = (config, e.span()) {
            let lines = config
                .char_indices()
                .collect::<Vec<_>>()
                .split(|(_, c)| *c == '\n')
                .filter_map(|line| {
                    line.iter()
                        .any(|(i, _)| span.contains(i))
                        .then_some(line.iter().map(|(_, c)| *c).collect::<String>())
                })
                .collect::<Vec<_>>();
            lines.join("\n")
        } else {
            String::new()
        };
        println!("{}\n---\n{}\n---", e.message(), position_lines);
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Config {
    editor: Vec<String>,
    theme: Theme,
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
            theme: Theme::Dark,
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

    pub fn theme(&self) -> Scheme {
        match self.theme {
            Theme::Dark => theme::dark::SCHEME,
            Theme::Light => theme::light::SCHEME,
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
