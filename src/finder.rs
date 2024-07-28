use regex::{Error, Regex};
use std::path::PathBuf;

pub struct Finder {
    files: Vec<PathBuf>,
    regex: Option<String>,
}

impl Finder {
    pub fn new(path: &PathBuf) -> Finder {
        Finder {
            files: crate::dir_pathes(path),
            regex: None,
        }
    }

    pub fn update(&mut self, pathes: Vec<PathBuf>) {
        self.files = pathes;
    }

    pub fn require(&self, i: usize) -> Option<&PathBuf> {
        if self.files.is_empty() || self.files.len() <= i {
            None
        } else {
            Some(&self.files[i])
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }

    pub fn search<S: AsRef<str>>(&mut self, new_reg: S) {
        self.regex = Some(String::from(new_reg.as_ref()));
    }

    pub fn is_search(&self) -> bool {
        self.regex.is_some()
    }

    pub fn cancel_search(&mut self) {
        self.regex = None;
    }

    pub fn regex(&self) -> Option<Result<Regex, Error>> {
        match self.regex {
            Some(ref s) => Some(Regex::new(s.as_str())),
            None => None,
        }
    }
}
