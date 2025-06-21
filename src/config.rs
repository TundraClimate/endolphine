use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::{
    io,
    path::{Path, PathBuf},
};

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

#[derive(Deserialize, Serialize)]
struct ConfigModel {
    editor: Vec<String>,
    theme: String,
}

pub struct Config {
    editor: Vec<String>,
    theme: Theme,
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

        Config {
            editor: model.editor,
            theme,
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
