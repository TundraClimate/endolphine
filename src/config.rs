mod init;
mod mapping;
mod theme;

use mapping::{KeymapConfig, KeymapRegistry};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use init::setup_local;
pub use theme::Theme;

pub fn file_path() -> PathBuf {
    let Some(home) = option_env!("HOME") else {
        panic!("Couldn't read the $HOME");
    };

    Path::new(home)
        .join(".config")
        .join("endolphine")
        .join("config.toml")
}

#[derive(Deserialize, Serialize)]
struct ConfigModel {
    editor: Vec<String>,
    theme: String,
    keymap: Option<KeymapConfig>,
}

pub fn parse_check(s: &str) -> Result<(), toml::de::Error> {
    toml::from_str::<ConfigModel>(s).map(|_| ())
}

pub fn handle_parse_err(config_read: String, e: toml::de::Error) {
    use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};

    let mut size_buf = 0usize;
    let span = e.span().unwrap();

    let err_lines = config_read
        .lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let before_len = size_buf;
            size_buf += line.len() + 1;

            (0..line.len())
                .any(|j| span.contains(&(j + before_len)))
                .then_some(format!("{} | {}\n", i + 1, line))
        })
        .collect::<String>();

    eprintln!(
        "{}{}",
        SetForegroundColor(Color::DarkCyan),
        SetAttribute(Attribute::Bold)
    );
    eprintln!("{:-^39}", "Invalid syntax detected");
    eprintln!("{}", e.message());
    eprintln!();
    eprintln!("{err_lines}");
    eprintln!("{}", "-".repeat(39));
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

pub struct Config {
    pub editor: Vec<String>,
    pub theme: Theme,
    pub keymaps: KeymapRegistry,
}

pub fn get() -> &'static Config {
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

        init::init_keymaps(&mut keymaps, &model.keymap);

        Config {
            editor: model.editor,
            theme,
            keymaps,
        }
    });

    &CONFIG
}
