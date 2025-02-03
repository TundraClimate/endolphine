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
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: vec!["vim"].into_iter().map(ToString::to_string).collect(),
        }
    }
}

impl Config {
    pub fn load() -> Config {
        file_path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|c| toml::from_str(&c).ok())
            .unwrap_or_default()
    }

    pub fn editor_command(&self) -> Option<std::process::Command> {
        let cmd = self.editor.first()?;
        let args = self.editor.iter().skip(1).collect::<Vec<_>>();
        let mut command = std::process::Command::new(cmd);
        command.args(args);
        Some(command)
    }
}
