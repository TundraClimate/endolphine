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

    pub fn cur_file(&self, cursor: usize) -> Option<&PathBuf> {
        if self.files.is_empty() {
            None
        } else {
            Some(&self.files[cursor])
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
