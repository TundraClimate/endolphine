use std::{
    ffi::OsString,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use tokio::{process::Command as TokioCommand, task};

pub fn trash_file(path: &PathBuf) {
    let name = crate::filename(&path);
    let home = dirs::home_dir().unwrap();
    if let (Some(path), Some(home)) = (path.to_str(), home.to_str()) {
        let trashed = PathBuf::from(format!("{home}/.local/share/Trash/files/{name}"));
        if trashed.exists() {
            if trashed.is_dir() {
                std::fs::remove_dir_all(&trashed).unwrap();
            } else if trashed.is_file() {
                std::fs::remove_file(&trashed).unwrap();
            }
        }
        Command::new("mv")
            .args([
                "-f",
                path,
                format!("{home}/.local/share/Trash/files/").as_str(),
            ])
            .spawn()
            .expect(format!("Command Error(cannot trashed) -> {}", path).as_str());
    }
}

pub fn move_file(from: &PathBuf, to: &PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("mv")
            .args(["-f", from, to])
            .spawn()
            .expect(format!("File move failed -> {}", from).as_str());
    }
}

pub fn copy_file(from: &PathBuf, to: &PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("cp")
            .args(["-f", "-r", from, to])
            .spawn()
            .expect(format!("File copy failed -> {}", from).as_str());
    }
}

pub fn create_file(path: &PathBuf) {
    if let Some(path) = path.to_str() {
        Command::new("touch")
            .args([path])
            .spawn()
            .expect(format!("File create failed -> {}", path).as_str());
    }
}

pub fn mkdir(path: &PathBuf) {
    if let Some(path) = path.to_str() {
        Command::new("mkdir")
            .args([path])
            .spawn()
            .expect(format!("Directory create failed -> {}", path).as_str());
    }
}

pub fn clip(pathes: Vec<&PathBuf>) -> io::Result<()> {
    let pathes = pathes
        .iter()
        .map(|p| format!("file://{}", p.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    let xclip = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed file copy to clipboard");
    if let Some(mut stdin) = xclip.stdin {
        stdin.write_all(&pathes.as_bytes())?;
    }
    Ok(())
}

pub fn clean_clipboard() -> io::Result<()> {
    let xclip = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed cleaning clipboard");
    if let Some(mut stdin) = xclip.stdin {
        stdin.write_all(b"")?;
    }
    Ok(())
}

pub fn clipboard() -> io::Result<String> {
    let output = Command::new("xclip")
        .arg("-selection")
        .arg("clipboard")
        .arg("-o")
        .output()?;

    let clipboard = String::from_utf8(output.stdout).expect("unparsed output");

    Ok(clipboard)
}

pub async fn editor(path: &PathBuf) -> io::Result<()> {
    TokioCommand::new("nvim")
        .args([path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub fn open_picture_viewer(path: &PathBuf) -> io::Result<()> {
    if let Some(path) = path.to_str() {
        Command::new("eog").args([path]).spawn()?;
    }
    Ok(())
}

pub fn is_video(path: &PathBuf) -> bool {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=codec_type",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute ffprobe");
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.trim() == "video"
}

pub fn open_video_viewer(path: &PathBuf) -> io::Result<()> {
    let _ = Command::new("vlc").args([path]).output()?;
    Ok(())
}

pub fn open_archiver(path: &PathBuf) -> io::Result<()> {
    Command::new("file-roller").args([path]).spawn()?;
    Ok(())
}

pub fn extract_zip(path: &PathBuf, outpath: OsString) -> io::Result<()> {
    let _ = Command::new("unzip")
        .arg("-o")
        .arg(path)
        .arg("-d")
        .arg(outpath)
        .output()?;
    Ok(())
}

pub fn extract_tgz(path: &PathBuf) -> io::Result<()> {
    let _ = Command::new("tar").arg("xzf").arg(path).output()?;
    Ok(())
}

pub fn open_pdf_viewer(path: &PathBuf) -> io::Result<()> {
    Command::new("evince").args([path]).spawn()?;
    Ok(())
}

pub fn zip(path: PathBuf) {
    task::spawn_blocking(move || {
        if let Some(archive) = path.parent() {
            let _ = Command::new("zip")
                .arg("-r")
                .arg(archive.join(format!("{}.zip", crate::filename(&path))))
                .arg(crate::filename(&path))
                .output()?;
        }
        Ok::<(), io::Error>(())
    });
}

pub fn tgz(path: PathBuf) {
    task::spawn_blocking(move || {
        if let Some(archive) = path.parent() {
            let _ = Command::new("tar")
                .arg("czf")
                .arg(archive.join(format!("{}.tar.gz", crate::filename(&path))))
                .arg(crate::filename(&path))
                .output()?;
        }
        Ok::<(), io::Error>(())
    });
}

pub fn gimp(path: &PathBuf) -> io::Result<()> {
    Command::new("gimp").args([path]).spawn()?;
    Ok(())
}
