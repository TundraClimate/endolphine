use crossterm::style::Color;

macro_rules! const_color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        pub const $name: Color = Color::Rgb {
            r: $r,
            g: $g,
            b: $b,
        };
    };

    ($name:ident, $color:expr) => {
        pub const $name: Color = Color::Rgb {
            r: $color,
            g: $color,
            b: $color,
        };
    };
}

const_color!(APP_BG, 60);
const_color!(APP_BG_DARK, 30);
const_color!(BAR, 150);
const_color!(BAR_DARK, 120);
const_color!(HEADER_CURRENT_PATH_ON_DARK, 150);
const_color!(HEADER_BAR_TEXT_DEFAULT, 40);
const_color!(HEADER_BAR_TEXT_LIGHT, 100);
const_color!(PERMISSION_TYPE, 30, 150, 230);
const_color!(PERMISSION_READ, 100, 220, 150);
const_color!(PERMISSION_WRITE, 240, 170, 70);
const_color!(PERMISSION_EXE, 250, 250, 60);
const_color!(PATH_NAME_FILE, 40, 220, 40);
const_color!(PATH_NAME_DIRECTORY, 40, 200, 200);
const_color!(PATH_NAME_SYMLINK, 200, 40, 200);
const_color!(PATH_NAME_BROKEN, 200, 0, 0);
const_color!(LAST_MODIFIED_TIME, 130, 70, 255);
const_color!(SELECTED, 235, 140, 0);
const_color!(UNDER_CURSOR, 85);
const_color!(INPUT_BG, 40, 40, 80);
const_color!(MENU_BG, 90);
const_color!(MENU_BG_DARK, 50);
const_color!(MENU_UNDER_CURSOR, 70);
const_color!(MENU_TAG_COLOR, 85, 240, 180);

pub fn app_bg() -> Color {
    if crate::global::menu().is_enabled() {
        APP_BG_DARK
    } else {
        APP_BG
    }
}

pub fn bar_color() -> Color {
    if crate::global::menu().is_enabled() {
        BAR_DARK
    } else {
        BAR
    }
}

pub fn menu_bg() -> Color {
    if crate::global::menu().is_enabled() {
        MENU_BG
    } else {
        MENU_BG_DARK
    }
}
