mod edit;
mod init;
mod mapping;
mod theme;

use edit::{EditConfig, HijackMapping};
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
    editor: Exec,
    theme: String,
    keymap: Option<KeymapConfig>,
    edit: Option<EditConfig>,
}

#[derive(Serialize)]
pub struct Exec {
    pub cmd: String,
    pub args: Vec<String>,
}

impl<'de> Deserialize<'de> for Exec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct StrOrVec;

        impl<'de> Visitor<'de> for StrOrVec {
            type Value = Exec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a value is available to string or [string]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let v = v.trim();

                if v.is_empty() {
                    return Err(serde::de::Error::custom("empty literal aren't available"));
                }

                Ok(Exec {
                    cmd: v.to_string(),
                    args: vec![],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut args = vec![];

                let Some(cmd) = seq.next_element::<String>()? else {
                    return Err(serde::de::Error::custom("first element aren't available"));
                };

                if cmd.is_empty() {
                    return Err(serde::de::Error::custom("empty command aren't available"));
                }

                while let Some(element) = seq.next_element::<String>()? {
                    let element = element.trim();

                    if element.is_empty() {
                        return Err(serde::de::Error::custom("empty literal aren't available"));
                    }

                    args.push(element.to_string());
                }

                Ok(Exec { cmd, args })
            }
        }

        deserializer.deserialize_any(StrOrVec)
    }
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
            editor: Exec {
                cmd: "vim".to_string(),
                args: vec![],
            },
            theme: "dark".to_string(),
            keymap: None,
            edit: None,
        }
    }
}

pub struct Config {
    pub editor: Exec,
    pub theme: Theme,
    pub keymaps: KeymapRegistry,
    pub hijack: HijackMapping,
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

        let hijack = HijackMapping::new(model.edit);

        Config {
            editor: model.editor,
            theme,
            keymaps,
            hijack,
        }
    });

    &CONFIG
}
