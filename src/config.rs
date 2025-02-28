use crate::{
    config,
    error::*,
    global,
    menu::MenuElement,
    theme::{self, Scheme, Theme},
};
use std::path::{Path, PathBuf};

global!(
    CONFIG<Config>,
    || {
        file_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str(&c).ok())
            .unwrap_or_default()
    },
    {
        fn try_load() -> Option<Result<Config, toml::de::Error>> {
            file_path()
                .and_then(|p| std::fs::read_to_string(p).ok())
                .map(|c| toml::from_str(&c))
        }
        pub fn load() -> &'static Config {
            &CONFIG
        }
    }
);

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

    if let Some(Err(e)) = try_load() {
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

    pub fn scheme(&self) -> Scheme {
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
