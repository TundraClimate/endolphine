use crate::app::App;
use image::io::Reader as ImageReader;
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

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
            files: crate::dir_pathes(None, value),
        }
    }
}

pub fn is_image(path: &PathBuf) -> io::Result<bool> {
    Ok(ImageReader::open(path)?
        .with_guessed_format()?
        .format()
        .is_some())
}

pub fn is_compressed(path: &PathBuf) -> io::Result<bool> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;

    let is_compressed = match &buffer {
        // gzip
        [0x1F, 0x8B, ..] => true,
        // zip
        [0x50, 0x4B, 0x03, 0x04] => true,
        // tar.gz
        [0x1F, 0x9D, ..] => true,
        // bzip2
        [0x42, 0x5A, 0x68, ..] => true,
        // xz
        [0xFD, 0x37, 0x7A, 0x58] => true,
        // 7z
        [0x37, 0x7A, 0xBC, 0xAF] => true,
        // rar
        [0x52, 0x61, 0x72, 0x21] => true,
        // lz4
        [0x04, 0x22, 0x4D, 0x18] => true,
        _ => false,
    };

    Ok(is_compressed)
}
