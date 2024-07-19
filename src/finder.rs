use regex::{Error, Regex};

pub struct Finder {
    regex: String,
}

impl Finder {
    pub fn new() -> Finder {
        Finder {
            regex: String::new(),
        }
    }

    pub fn search<S: AsRef<str>>(&mut self, new_reg: S) {
        self.regex = String::from(new_reg.as_ref());
    }

    pub fn regex(&self) -> Result<Regex, Error> {
        Regex::new(self.regex.as_str())
    }
}
