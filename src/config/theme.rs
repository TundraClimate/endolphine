use crossterm::style::Color;
use serde::Deserialize;
use std::{
    io,
    path::{Path, PathBuf},
};

pub(super) fn dir_path() -> PathBuf {
    let Some(home) = option_env!("HOME") else {
        panic!("Couldn't read the $HOME");
    };

    Path::new(home)
        .join(".config")
        .join("endolphine")
        .join("theme")
}

#[derive(Clone, Copy)]
pub struct HexColor(Color);

impl From<HexColor> for Color {
    fn from(value: HexColor) -> Self {
        value.0
    }
}

fn rgb(t: &str) -> Result<Color, String> {
    let (true, Ok(r), Ok(g), Ok(b)) = (
        t.len() == 7 && t.starts_with("#"),
        u8::from_str_radix(&t[1..3], 16),
        u8::from_str_radix(&t[3..5], 16),
        u8::from_str_radix(&t[5..], 16),
    ) else {
        return Err(format!("Invalid token: {t}"));
    };

    Ok(Color::Rgb { r, g, b })
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;
        rgb(&s).map_err(Error::custom).map(HexColor)
    }
}

#[derive(Deserialize)]
pub struct Theme {
    pub app_fg: HexColor,
    pub app_bg: HexColor,
    pub bar_fg: HexColor,
    pub bar_fg_light: HexColor,
    pub bar_bg: HexColor,
    pub item_bg_cursor: HexColor,
    pub item_bg_select: HexColor,
    pub item_broken: HexColor,
    pub item_dir: HexColor,
    pub item_file: HexColor,
    pub item_symlink: HexColor,
    pub item_sidemenu: HexColor,
    pub item_parts_bsize: HexColor,
    pub item_parts_lmd: HexColor,
    pub perm_ty: HexColor,
    pub perm_r: HexColor,
    pub perm_w: HexColor,
    pub perm_x: HexColor,
    pub pwd_view: HexColor,
    pub pwd_pickouted: HexColor,
    pub search_surround: HexColor,
    pub mode_normal: HexColor,
    pub mode_visual: HexColor,
    pub mode_input: HexColor,
    pub mode_search: HexColor,
    pub mode_menu: HexColor,
}

pub async fn download_official_theme(name: &str) -> io::Result<()> {
    use std::fs;

    log::info!("The official theme downloading");

    let dest_path = dir_path().join(format!("{name}.toml"));

    if dest_path.exists() {
        panic!("{name} is already exists");
    }

    // TODO FIX URL to master
    let official_url = format!(
        "https://raw.githubusercontent.com/TundraClimate/endolphine/refs/heads/feature/render/theme/{name}.toml",
    );

    let Ok(res) = reqwest::get(official_url).await else {
        panic!("Cannot access to official repository");
    };

    if res.status().is_client_error() {
        panic!("\"{name}\" is invalid token");
    }

    let Ok(bytes) = res.bytes().await else {
        panic!("Couldn't read bytes");
    };

    fs::write(dest_path, bytes)?;

    Ok(())
}

pub async fn download_unofficial_theme(url: &str) -> io::Result<()> {
    use std::fs;

    log::info!("The unofficial theme downloading");

    let Some(name) = url
        .split('/')
        .next_back()
        .and_then(|name| Path::new(name).file_stem())
        .and_then(|name| name.to_str())
    else {
        panic!("{url} is not valid");
    };

    let dest_path = dir_path().join(format!("{name}.toml"));

    if dest_path.exists() {
        panic!("{name} is already exists");
    }

    let Ok(res) = reqwest::get(url).await else {
        panic!("Cannot access to {url}");
    };

    if res.status().is_client_error() {
        panic!("'{name}' is invalid url");
    }

    let Ok(bytes) = res.bytes().await else {
        panic!("Couldn't read bytes");
    };

    let Ok(content) = str::from_utf8(&bytes) else {
        panic!("Invalid token found");
    };

    let is_valid = toml::from_str::<Theme>(content).is_ok();

    if !is_valid {
        panic!("The {url} content is cannot be parse");
    }

    fs::write(dest_path, bytes)?;

    Ok(())
}
