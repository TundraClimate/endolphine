use crate::app::App;
use std::path::PathBuf;

pub struct FileManager {
    files: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(app: &App) -> FileManager {
        FileManager {
            files: crate::dir_pathes(Some(app), &app.path),
        }
    }

    pub fn require(&self, i: usize) -> Option<&PathBuf> {
        if self.files.is_empty() {
            None
        } else {
            Some(&self.files[i])
        }
    }

    pub fn len(&self) -> usize {
        self.files.len()
    }
}

impl From<&PathBuf> for FileManager {
    fn from(value: &PathBuf) -> Self {
        FileManager {
            files: crate::dir_pathes(None, value),
        }
    }
}
