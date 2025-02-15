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
    pub rm: RmConfig,
    pub paste: PasteConfig,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RmConfig {
    pub no_enter: bool,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PasteConfig {
    copied_suffix: String,
    pub force_mode: bool,
    pub default_overwrite: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: vec!["vim"].into_iter().map(ToString::to_string).collect(),
            rm: RmConfig { no_enter: false },
            paste: PasteConfig {
                copied_suffix: String::from("_Copy"),
                force_mode: true,
                default_overwrite: true,
            },
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
