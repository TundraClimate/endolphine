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

struct HexColor(Color);

fn rgb(t: &str) -> Result<Color, String> {
    let (true, Ok(r), Ok(g), Ok(b)) = (
        t.len() == 7 && t.starts_with("#"),
        u8::from_str_radix(&t[1..3], 16),
        u8::from_str_radix(&t[3..5], 16),
        u8::from_str_radix(&t[5..], 16),
    ) else {
        return Err(format!("Invalid token: {}", t));
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
    app_fg: HexColor,
    app_bg: HexColor,
    bar_fg: HexColor,
    bar_fg_light: HexColor,
    bar_bg: HexColor,
    item_bg_cursor: HexColor,
    item_bg_select: HexColor,
    item_broken: HexColor,
    item_dir: HexColor,
    item_file: HexColor,
    item_symlink: HexColor,
    item_sidemenu: HexColor,
    item_parts_bsize: HexColor,
    item_parts_lmd: HexColor,
    perm_ty: HexColor,
    perm_r: HexColor,
    perm_w: HexColor,
    perm_x: HexColor,
    pwd_view: HexColor,
    pwd_pickouted: HexColor,
    search_surround: HexColor,
}

pub(super) async fn download_official_theme(name: &str) -> io::Result<()> {
    use std::fs;

    // TODO FIX URL to master
    let official_url = format!(
        "https://raw.githubusercontent.com/TundraClimate/endolphine/refs/heads/feature/render/theme/{}.toml",
        name
    );

    let Ok(res) = reqwest::get(official_url).await else {
        panic!("Cannot access to official repository");
    };

    if res.status().is_client_error() {
        panic!("\"{}\" is invalid token", name);
    }

    let Ok(bytes) = res.bytes().await else {
        panic!("Couldn't read bytes");
    };

    let dest_path = dir_path().join(format!("{}.toml", name));

    fs::write(dest_path, bytes)?;

    Ok(())
}
