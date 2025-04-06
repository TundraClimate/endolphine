pub struct Keymap(Vec<Key>);

impl Keymap {
    pub fn nth(&self, index: usize) -> Option<&Key> {
        self.0.get(index)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Clone for Keymap {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl From<&str> for Keymap {
    fn from(value: &str) -> Self {
        value.parse().unwrap_or(Keymap(vec![]))
    }
}

impl std::str::FromStr for Keymap {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut in_tag = false;
        let mut buf = String::new();
        let mut keys: Vec<Key> = vec![];

        for c in s.chars() {
            if c == '<' {
                in_tag = true;
            }

            if in_tag {
                buf.push(c);
            } else {
                keys.push(c.to_string().parse()?)
            }

            if c == '>' {
                in_tag = false;
                keys.push(buf.parse()?);
                buf.clear();
            }
        }

        if in_tag {
            return Err(String::from("invalid format"));
        }

        Ok(Keymap(keys))
    }
}

impl std::fmt::Display for Keymap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .fold(String::new(), |acc, k| format!("{}{}", acc, k))
        )
    }
}

impl<'de> serde::Deserialize<'de> for Keymap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialize = String::deserialize(deserializer)?;
        deserialize.parse().map_err(serde::de::Error::custom)
    }
}

impl serde::Serialize for Keymap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Copy)]
pub struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Key {{ code: {}, modifiers: KeyModifiers({}) }}",
            self.code as u8, self.modifiers.0 as u8
        )
    }
}

impl Key {
    pub fn from_keyevent(e: &crossterm::event::KeyEvent) -> Key {
        let code = match e.code {
            crossterm::event::KeyCode::Backspace => KeyCode::Backspace,
            crossterm::event::KeyCode::Tab => KeyCode::Tab,
            crossterm::event::KeyCode::Enter => KeyCode::Enter,
            crossterm::event::KeyCode::Esc => KeyCode::Esc,
            crossterm::event::KeyCode::Char(' ') => KeyCode::Space,
            crossterm::event::KeyCode::Char('"') => KeyCode::QuotationMark,
            crossterm::event::KeyCode::Char('#') => KeyCode::NumberSign,
            crossterm::event::KeyCode::Char('$') => KeyCode::DollarSign,
            crossterm::event::KeyCode::Char('%') => KeyCode::PercentSign,
            crossterm::event::KeyCode::Char('(') => KeyCode::LeftParenthesis,
            crossterm::event::KeyCode::Char(')') => KeyCode::RightParenthesis,
            crossterm::event::KeyCode::Char('*') => KeyCode::Asterisk,
            crossterm::event::KeyCode::Char('+') => KeyCode::PlusSign,
            crossterm::event::KeyCode::Char(',') => KeyCode::Comma,
            crossterm::event::KeyCode::Char('-') => KeyCode::HyphenMinus,
            crossterm::event::KeyCode::Char('.') => KeyCode::FullStop,
            crossterm::event::KeyCode::Char('/') => KeyCode::Solidus,
            crossterm::event::KeyCode::Char('0') => KeyCode::Digit0,
            crossterm::event::KeyCode::Char('1') => KeyCode::Digit1,
            crossterm::event::KeyCode::Char('2') => KeyCode::Digit2,
            crossterm::event::KeyCode::Char('3') => KeyCode::Digit3,
            crossterm::event::KeyCode::Char('4') => KeyCode::Digit4,
            crossterm::event::KeyCode::Char('5') => KeyCode::Digit5,
            crossterm::event::KeyCode::Char('6') => KeyCode::Digit6,
            crossterm::event::KeyCode::Char('7') => KeyCode::Digit7,
            crossterm::event::KeyCode::Char('8') => KeyCode::Digit8,
            crossterm::event::KeyCode::Char('9') => KeyCode::Digit9,
            crossterm::event::KeyCode::Char(':') => KeyCode::Colon,
            crossterm::event::KeyCode::Char(';') => KeyCode::Semicolon,
            crossterm::event::KeyCode::Char('?') => KeyCode::QuestionMark,
            crossterm::event::KeyCode::Char('@') => KeyCode::CommercialAt,
            crossterm::event::KeyCode::Char('a') => KeyCode::A,
            crossterm::event::KeyCode::Char('b') => KeyCode::B,
            crossterm::event::KeyCode::Char('c') => KeyCode::C,
            crossterm::event::KeyCode::Char('d') => KeyCode::D,
            crossterm::event::KeyCode::Char('e') => KeyCode::E,
            crossterm::event::KeyCode::Char('f') => KeyCode::F,
            crossterm::event::KeyCode::Char('g') => KeyCode::G,
            crossterm::event::KeyCode::Char('h') => KeyCode::H,
            crossterm::event::KeyCode::Char('i') => KeyCode::I,
            crossterm::event::KeyCode::Char('j') => KeyCode::J,
            crossterm::event::KeyCode::Char('k') => KeyCode::K,
            crossterm::event::KeyCode::Char('l') => KeyCode::L,
            crossterm::event::KeyCode::Char('m') => KeyCode::M,
            crossterm::event::KeyCode::Char('n') => KeyCode::N,
            crossterm::event::KeyCode::Char('o') => KeyCode::O,
            crossterm::event::KeyCode::Char('p') => KeyCode::P,
            crossterm::event::KeyCode::Char('q') => KeyCode::Q,
            crossterm::event::KeyCode::Char('r') => KeyCode::R,
            crossterm::event::KeyCode::Char('s') => KeyCode::S,
            crossterm::event::KeyCode::Char('t') => KeyCode::T,
            crossterm::event::KeyCode::Char('u') => KeyCode::U,
            crossterm::event::KeyCode::Char('v') => KeyCode::V,
            crossterm::event::KeyCode::Char('w') => KeyCode::W,
            crossterm::event::KeyCode::Char('x') => KeyCode::X,
            crossterm::event::KeyCode::Char('y') => KeyCode::Y,
            crossterm::event::KeyCode::Char('z') => KeyCode::Z,
            crossterm::event::KeyCode::Char('A') => KeyCode::A,
            crossterm::event::KeyCode::Char('B') => KeyCode::B,
            crossterm::event::KeyCode::Char('C') => KeyCode::C,
            crossterm::event::KeyCode::Char('D') => KeyCode::D,
            crossterm::event::KeyCode::Char('E') => KeyCode::E,
            crossterm::event::KeyCode::Char('F') => KeyCode::F,
            crossterm::event::KeyCode::Char('G') => KeyCode::G,
            crossterm::event::KeyCode::Char('H') => KeyCode::H,
            crossterm::event::KeyCode::Char('I') => KeyCode::I,
            crossterm::event::KeyCode::Char('J') => KeyCode::J,
            crossterm::event::KeyCode::Char('K') => KeyCode::K,
            crossterm::event::KeyCode::Char('L') => KeyCode::L,
            crossterm::event::KeyCode::Char('M') => KeyCode::M,
            crossterm::event::KeyCode::Char('N') => KeyCode::N,
            crossterm::event::KeyCode::Char('O') => KeyCode::O,
            crossterm::event::KeyCode::Char('P') => KeyCode::P,
            crossterm::event::KeyCode::Char('Q') => KeyCode::Q,
            crossterm::event::KeyCode::Char('R') => KeyCode::R,
            crossterm::event::KeyCode::Char('S') => KeyCode::S,
            crossterm::event::KeyCode::Char('T') => KeyCode::T,
            crossterm::event::KeyCode::Char('U') => KeyCode::U,
            crossterm::event::KeyCode::Char('V') => KeyCode::V,
            crossterm::event::KeyCode::Char('W') => KeyCode::W,
            crossterm::event::KeyCode::Char('X') => KeyCode::X,
            crossterm::event::KeyCode::Char('Y') => KeyCode::Y,
            crossterm::event::KeyCode::Char('Z') => KeyCode::Z,
            crossterm::event::KeyCode::Char('[') => KeyCode::LeftSquareBracket,
            crossterm::event::KeyCode::Char('\\') => KeyCode::ReverseSolidas,
            crossterm::event::KeyCode::Char(']') => KeyCode::RightSquareBracket,
            crossterm::event::KeyCode::Char('^') => KeyCode::CircumflexAccent,
            crossterm::event::KeyCode::Char('_') => KeyCode::LowLine,
            crossterm::event::KeyCode::Char('{') => KeyCode::LeftCurlyBracket,
            crossterm::event::KeyCode::Char('}') => KeyCode::RightCurlyBracket,
            _ => KeyCode::None,
        };

        let mut modifiers = KeyModifier::None;

        if e.modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
            modifiers = modifiers | KeyModifier::Shift;
        }

        if e.modifiers.contains(crossterm::event::KeyModifiers::ALT) {
            modifiers = modifiers | KeyModifier::Alt;
        }

        if e.modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            modifiers = modifiers | KeyModifier::Control;
        }

        Key {
            code,
            modifiers: KeyModifiers(modifiers),
        }
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code && self.modifiers == other.modifiers
    }
}

impl std::str::FromStr for Key {
    type Err = String;

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        if !tag.is_ascii() {
            return Err(String::from("unsupported key format"));
        }

        if tag.is_empty() {
            return Err(String::from("format is empty"));
        }

        if tag.len() == 1 {
            let Ok(tag) = char::from_str(tag) else {
                return Err(String::from("unsupported key format"));
            };

            let modifier = if tag.is_ascii_uppercase() {
                KeyModifier::Shift
            } else {
                KeyModifier::None
            };

            let tag = tag.to_ascii_uppercase();

            let code = match tag {
                'A'..='Z' => KeyCode::from_ascii(tag as u8),
                '"' | '#' | '$' | '%' | '(' | ')' | '*' | '+' | '?' | '_' | '{' | '}' | '-'
                | '[' | ']' | ',' | '.' | '/' | ':' | ';' | '@' | '\\' | '^' => {
                    KeyCode::from_ascii(tag as u8)
                }

                tag if tag.is_ascii_digit() => KeyCode::from_ascii(tag as u8),

                _ => return Err(String::from("unsupported key format")),
            };

            return Ok(Key {
                code,
                modifiers: KeyModifiers(modifier),
            });
        }

        let is_special = tag.starts_with("<") && tag.ends_with(">");

        if !is_special || tag.len() == 2 {
            return Err(String::from("unsupported key format"));
        }

        let is_modded = tag.chars().nth(2).is_some_and(|c| c == '-');
        let base = if is_modded {
            &tag[3..tag.len() - 1]
        } else {
            &tag[1..tag.len() - 1]
        };
        let modifier = if is_modded {
            match tag.chars().nth(1).map(|c| c.to_ascii_lowercase()) {
                Some('a') => KeyModifier::Alt,
                Some('c') => KeyModifier::Control,
                Some('s') => KeyModifier::Shift,
                _ => KeyModifier::None,
            }
        } else {
            KeyModifier::None
        };

        if base.len() == 1 {
            let mut key = Key::from_str(base)?;

            key.modifiers = KeyModifiers(key.modifiers.0 | modifier);

            return Ok(key);
        }

        let code = match base.to_lowercase().as_str() {
            "enter" | "cr" => KeyCode::Enter,
            "tab" => KeyCode::Tab,
            "esc" => KeyCode::Esc,
            "leader" | "space" => KeyCode::Space,
            "bs" => KeyCode::Backspace,
            _ => return Err(String::from("unsupported key format")),
        };

        Ok(Key {
            code,
            modifiers: KeyModifiers(modifier),
        })
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", 'blk: {
            if self.code == KeyCode::None {
                break 'blk String::new();
            }

            let is_special = matches!(
                self.code,
                KeyCode::Enter | KeyCode::Tab | KeyCode::Esc | KeyCode::Space | KeyCode::Backspace
            );
            let is_modded = self.modifiers.is_alt() || self.modifiers.is_ctrl();
            let is_shift = self.modifiers.is_shift();
            let is_alpha = matches!(self.code as u8, 65..=90);

            let code = match &self.code {
                KeyCode::Enter => "CR",
                KeyCode::Tab => "TAB",
                KeyCode::Esc => "ESC",
                KeyCode::Space => "SPACE",
                KeyCode::Backspace => "BS",

                keycode if !is_shift && is_alpha => {
                    &format!("{}", keycode.as_ascii().to_ascii_lowercase())
                }

                keycode => &format!("{}", keycode.as_ascii()),
            };

            let code = if self.modifiers.is_alt() {
                &format!("a-{}", code)
            } else if self.modifiers.is_ctrl() {
                &format!("c-{}", code)
            } else if is_shift && !is_alpha {
                &format!("s-{}", code)
            } else {
                &code.to_string()
            };

            if is_special || is_modded || is_shift && !is_alpha {
                format!("<{}>", code)
            } else {
                code.to_string()
            }
        })
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
enum KeyCode {
    Backspace = 8,
    Tab = 9,
    Enter = 13,
    Esc = 27,
    Space = 32,
    QuotationMark = 34,
    NumberSign = 35,
    DollarSign = 36,
    PercentSign = 37,
    LeftParenthesis = 40,
    RightParenthesis = 41,
    Asterisk = 42,
    PlusSign = 43,
    Comma = 44,
    HyphenMinus = 45,
    FullStop = 46,
    Solidus = 47,
    Digit0 = 48,
    Digit1 = 49,
    Digit2 = 50,
    Digit3 = 51,
    Digit4 = 52,
    Digit5 = 53,
    Digit6 = 54,
    Digit7 = 55,
    Digit8 = 56,
    Digit9 = 57,
    Colon = 58,
    Semicolon = 59,
    QuestionMark = 63,
    CommercialAt = 64,
    A = 65,
    B = 66,
    C = 67,
    D = 68,
    E = 69,
    F = 70,
    G = 71,
    H = 72,
    I = 73,
    J = 74,
    K = 75,
    L = 76,
    M = 77,
    N = 78,
    O = 79,
    P = 80,
    Q = 81,
    R = 82,
    S = 83,
    T = 84,
    U = 85,
    V = 86,
    W = 87,
    X = 88,
    Y = 89,
    Z = 90,
    LeftSquareBracket = 91,
    ReverseSolidas = 92,
    RightSquareBracket = 93,
    CircumflexAccent = 94,
    LowLine = 95,
    LeftCurlyBracket = 123,
    RightCurlyBracket = 125,
    None = 0,
}

impl KeyCode {
    fn from_ascii(ascii: u8) -> KeyCode {
        unsafe { std::mem::transmute(ascii) }
    }

    fn as_ascii(&self) -> char {
        std::char::from_u32(*self as u32).unwrap()
    }
}

impl PartialEq for KeyCode {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}

#[derive(Clone, Copy)]
struct KeyModifiers(KeyModifier);

impl KeyModifiers {
    pub fn is_shift(&self) -> bool {
        self.0 & KeyModifier::Shift == KeyModifier::Shift
    }

    pub fn is_alt(&self) -> bool {
        self.0 & KeyModifier::Alt == KeyModifier::Alt
    }

    pub fn is_ctrl(&self) -> bool {
        self.0 & KeyModifier::Control == KeyModifier::Control
    }
}

impl PartialEq for KeyModifiers {
    fn eq(&self, other: &Self) -> bool {
        self.0 as u8 == other.0 as u8
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
enum KeyModifier {
    Shift = 0b0001,
    Control = 0b0010,
    Alt = 0b0100,
    None = 0b0000,
}

impl std::ops::BitAnd for KeyModifier {
    type Output = KeyModifier;

    fn bitand(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute(self as u8 & rhs as u8) }
    }
}

impl std::ops::BitOr for KeyModifier {
    type Output = KeyModifier;

    fn bitor(self, rhs: Self) -> Self::Output {
        unsafe { std::mem::transmute(self as u8 | rhs as u8) }
    }
}
