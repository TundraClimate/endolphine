use std::path::PathBuf;

pub struct FileManager {
    files: Vec<PathBuf>,
}

impl FileManager {
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
}

impl From<&PathBuf> for FileManager {
    fn from(value: &PathBuf) -> Self {
        FileManager {
            files: crate::dir_pathes(value),
        }
    }
}
