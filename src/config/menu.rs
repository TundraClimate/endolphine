use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Deserialize, Serialize)]
pub(super) struct MenuConfig {
    pub(super) items: Vec<MenuElement>,
}

impl Default for MenuConfig {
    fn default() -> Self {
        let home_path = option_env!("HOME").unwrap_or("/root");
        let dls_path = format!("{home_path}/Downloads");
        let desktop_path = format!("{home_path}/Desktop");

        Self {
            items: vec![
                MenuElement::new("Home", home_path),
                MenuElement::new("Downloads", dls_path),
                MenuElement::new("Desktop", desktop_path),
            ],
        }
    }
}

pub struct MenuElement {
    tag: String,
    path: PathBuf,
}

impl MenuElement {
    fn new<S: Display, P: AsRef<Path>>(tag: S, path: P) -> Self {
        Self {
            tag: tag.to_string(),
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Serialize for MenuElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}:{}", self.tag, self.path.to_string_lossy()))
    }
}

impl<'de> Deserialize<'de> for MenuElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        match s.trim().split_once(":") {
            Some((tag, path)) => Ok(MenuElement::new(tag, path)),
            None => {
                let path = Path::new(&s);

                match path.file_name().and_then(|name| name.to_str()) {
                    Some(tag) => Ok(MenuElement::new(tag, path)),
                    None => Err(serde::de::Error::custom("Filename cannot find")),
                }
            }
        }
    }
}
