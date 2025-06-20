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

pub fn setup_local() -> io::Result<()> {
    use std::fs;

    let config_path = file_path();

    if !config_path.exists() {
        let parent = config_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        let theme_dir = parent.join("theme");

        if !theme_dir.exists() {
            fs::create_dir_all(&theme_dir)?;
        }

        if theme_dir.read_dir().is_ok_and(|dir| dir.count() == 0) {
            // donwload_dark_theme()
            // https://raw.githubusercontent.com/TundraClimate/endolphine/refs/heads/master/theme/dark.toml
        }

        fs::write(config_path, b"")?;
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
}

pub fn get() -> &'static Config {
    use std::{fs, sync::LazyLock};

    static CONFIG: LazyLock<Config> = LazyLock::new(|| {
        let model = fs::read_to_string(file_path())
            .ok()
            .and_then(|config| toml::from_str::<ConfigModel>(&config).ok())
            .unwrap_or_default();

        Config {
            editor: model.editor,
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
