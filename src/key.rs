struct Keymap(Vec<Key>);

impl<'de> serde::Deserialize<'de> for Keymap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let deserialize = String::deserialize(deserializer)?;
        deserialize.parse().map_err(serde::de::Error::custom)
    }
}

struct Key {
    code: KeyCode,
    modifiers: KeyModifiers,
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

impl std::str::FromStr for Key {
    type Err = String;

    fn from_str(tag: &str) -> Result<Self, Self::Err> {
        if !tag.is_ascii() {
            return Err(String::from("not supported format"));
        }

        let is_special_tag = tag.len() > 3 && tag.starts_with("<") && tag.ends_with(">");
        let tag = tag.trim_start_matches('<').trim_end_matches('>');

        if is_special_tag {
            let is_shift = tag.chars().nth(2).is_some_and(|c| c.is_ascii_uppercase());
            let tag = tag.to_ascii_lowercase();
            let (tag, modifier) = if let Some(tag) = tag.strip_prefix("s-") {
                (tag, KeyModifier::Shift)
            } else if let Some(tag) = tag.strip_prefix("a-") {
                (tag, KeyModifier::Alt)
            } else if let Some(tag) = tag.strip_prefix("c-") {
                (tag, KeyModifier::Control)
            } else {
                (tag.as_str(), KeyModifier::None)
            };

            match tag {
                "enter" | "cr" => Ok(Key {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers(modifier),
                }),
                "tab" => Ok(Key {
                    code: KeyCode::Tab,
                    modifiers: KeyModifiers(modifier),
                }),
                "esc" => Ok(Key {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers(modifier),
                }),
                "leader" | "space" => Ok(Key {
                    code: KeyCode::Space,
                    modifiers: KeyModifiers(modifier),
                }),
                "bs" => Ok(Key {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers(modifier),
                }),
                "\"" | "#" | "$" | "%" | "(" | ")" | "*" | "+" | "?" | "_" | "{" | "}" | "-"
                | "[" | "]" | "," | "." | "/" | ":" | ";" | "@" | "\\" | "^" => {
                    let code = unsafe {
                        std::mem::transmute::<u8, KeyCode>(tag.chars().nth(0).unwrap() as u8)
                    };
                    Ok(Key {
                        code,
                        modifiers: KeyModifiers(modifier),
                    })
                }

                tag if tag.chars().nth(0).is_some_and(|c| c.is_ascii_digit()) => {
                    let code = unsafe {
                        std::mem::transmute::<u8, KeyCode>(tag.chars().nth(0).unwrap() as u8)
                    };
                    Ok(Key {
                        code,
                        modifiers: KeyModifiers(modifier),
                    })
                }

                tag if tag.chars().nth(0).is_some_and(|c| c.is_ascii_lowercase()) => {
                    let code = unsafe {
                        std::mem::transmute::<u8, KeyCode>(tag.chars().nth(0).unwrap() as u8 - 32)
                    };
                    if is_shift {
                        Ok(Key {
                            code,
                            modifiers: KeyModifiers(KeyModifier::Shift | modifier),
                        })
                    } else {
                        Ok(Key {
                            code,
                            modifiers: KeyModifiers(modifier),
                        })
                    }
                }

                _ => Err(String::from("invalid tag format")),
            }
        } else {
            let tag = tag.chars().nth(0).unwrap();
            let modifier = if tag.is_ascii_uppercase() {
                KeyModifier::Shift
            } else {
                KeyModifier::None
            };
            let tag = tag.to_ascii_lowercase();

            match tag {
                'a'..='z' => Ok(Key {
                    code: unsafe { std::mem::transmute::<u8, KeyCode>(tag as u8 - 32) },
                    modifiers: KeyModifiers(modifier),
                }),

                tag if tag.is_ascii_digit() => {
                    let code = unsafe { std::mem::transmute::<u8, KeyCode>(tag as u8) };
                    Ok(Key {
                        code,
                        modifiers: KeyModifiers(modifier),
                    })
                }

                '"' | '#' | '$' | '%' | '(' | ')' | '*' | '+' | '?' | '_' | '{' | '}' | '-'
                | '[' | ']' | ',' | '.' | '/' | ':' | ';' | '@' | '\\' | '^' => {
                    let code = unsafe { std::mem::transmute::<u8, KeyCode>(tag as u8) };
                    Ok(Key {
                        code,
                        modifiers: KeyModifiers(KeyModifier::None),
                    })
                }

                _ => Err(String::from("invalid format")),
            }
        }
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
}

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

    pub fn is_none(&self) -> bool {
        self.0 == KeyModifier::None
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
