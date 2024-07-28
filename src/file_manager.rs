use crate::{app::App, command, ui};
use image::io::Reader as ImageReader;
use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use tokio::task;

pub struct FileManager {
    files: Vec<PathBuf>,
}

impl FileManager {
    pub fn new(app: &App) -> FileManager {
        FileManager {
            files: crate::rows(app, &app.path),
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
            files: crate::dir_pathes(value),
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

pub fn extract_from_archive(path: PathBuf) {
    task::spawn_blocking(move || {
        let mut file = File::open(&path)?;
        let mut buffer = [0; 4];
        file.read_exact(&mut buffer)?;

        match &buffer {
            [0x50, 0x4B, 0x03, 0x04] => extract_zip(&path)?,
            [0x1F, 0x8B, ..] => extract_tgz(&path)?,
            _ => {}
        }
        Ok::<(), io::Error>(())
    });
}

fn extract_zip(path: &PathBuf) -> io::Result<()> {
    let outpath = path
        .file_stem()
        .map(|s| s.to_os_string())
        .unwrap_or("out".into());
    if let Some(parent) = path.parent() {
        if !parent.join(&outpath).exists() {
            command::extract_zip(&path, outpath)?;
            ui::log(format!(
                "Archive \"{}\" has been extracted",
                crate::filename(path)
            ))?;
        } else {
            ui::log(format!("Could not extract {}", crate::filename(&path)))?;
        }
    }
    Ok(())
}

fn extract_tgz(path: &PathBuf) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        let outpath = path
            .file_stem()
            .map(|s| PathBuf::from(s).file_stem().unwrap().to_os_string())
            .unwrap_or("out".into());
        if !parent.join(&outpath).exists() {
            command::extract_tgz(&path)?;
            ui::log(format!(
                "Archive \"{}\" has been extracted",
                crate::filename(path)
            ))?;
        } else {
            ui::log(format!("Could not extract {}", crate::filename(&path)))?;
        }
    }
    Ok(())
}

pub fn zip(path: PathBuf) {
    tokio::spawn(async move {
        command::zip(&path).await?;
        ui::log(format!(
            "Created an archive for \"{}\"",
            crate::filename(&path)
        ))?;
        Ok::<(), io::Error>(())
    });
}

pub fn tgz(path: PathBuf) {
    tokio::spawn(async move {
        command::tgz(&path).await?;
        ui::log(format!(
            "Created an archive for \"{}\"",
            crate::filename(&path)
        ))?;
        Ok::<(), io::Error>(())
    });
}
