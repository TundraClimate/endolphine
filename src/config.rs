mod delete;
mod edit;
mod init;
mod mapping;
mod menu;
mod paste;
mod theme;

use delete::DeleteConfig;
use edit::{EditConfig, HijackMapping};
use mapping::{KeymapConfig, KeymapRegistry};
use menu::{MenuConfig, MenuElement};
use paste::PasteConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub use init::setup_local;
pub use theme::{Theme, download_official_theme, download_unofficial_theme};

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
    theme: String,
    native_cb: bool,
    keymap: Option<KeymapConfig>,
    delete: DeleteConfig,
    paste: PasteConfig,
    edit: EditConfig,
    menu: MenuConfig,
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

    log::error!("The new configuration was unsuccessfully parsed");

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

pub fn print_success_message() {
    use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};

    let path = file_path();

    log::info!("The new configuration was successfully parsed");

    println!(
        "{}{}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::Green),
    );
    println!("{}", "-".repeat(39));
    println!(" New config save successful");
    println!();
    println!(" > {}", path.to_string_lossy());
    println!("{}", "-".repeat(39));
}

impl Default for ConfigModel {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            native_cb: false,
            keymap: None,
            delete: DeleteConfig::default(),
            paste: PasteConfig::default(),
            edit: EditConfig::default(),
            menu: MenuConfig::default(),
        }
    }
}

pub struct Config {
    pub theme: Theme,
    pub native_cb: bool,
    pub keymaps: KeymapRegistry,
    pub hijack: HijackMapping,
    pub delete_to_temp: bool,
    pub delete_with_yank: bool,
    pub paste_similar_suffix: String,
    pub paste_is_overwrite: bool,
    pub menu_elements: Vec<MenuElement>,
}

pub fn get() -> &'static Config {
    use std::{fs, sync::LazyLock};

    static CONFIG: LazyLock<Config> = LazyLock::new(|| {
        log::info!("The config initialize");

        log::info!("Load config file");

        let model = match fs::read_to_string(file_path())
            .ok()
            .and_then(|config| toml::from_str::<ConfigModel>(&config).ok())
        {
            Some(config) => config,
            None => {
                log::warn!("The configuration cannot load");
                log::warn!("Instead default configuration");
                ConfigModel::default()
            }
        };
        let theme_path = theme::dir_path().join(format!("{}.toml", model.theme));

        log::info!("Load application theme");

        let Some(theme) = fs::read_to_string(theme_path)
            .ok()
            .and_then(|theme| toml::from_str::<Theme>(&theme).ok())
        else {
            panic!("Failed to load theme file");
        };

        let native_cb = model.native_cb;

        let mut keymaps = KeymapRegistry::new();

        log::info!("Initialize keymaps");

        init::init_keymaps(&model, &mut keymaps, &model.keymap);

        let hijack = HijackMapping::new(model.edit);

        let menu_elements = model.menu.items;

        let delete_to_temp = model.delete.put_to_temp;
        let delete_with_yank = model.delete.with_yank;

        let paste_similar_suffix = model.paste.copied_suffix;
        let paste_is_overwrite = model.paste.is_overwrite;

        log::info!("Initialize success");

        Config {
            theme,
            native_cb,
            keymaps,
            hijack,
            delete_to_temp,
            delete_with_yank,
            paste_similar_suffix,
            paste_is_overwrite,
            menu_elements,
        }
    });

    &CONFIG
}
