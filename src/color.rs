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

const_color!(APP_BG, 90);
const_color!(DEFAULT_BAR, 180);
const_color!(HEADER_CURRENT_PATH_ON_DARK, 150);
