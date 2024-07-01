use std::{
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
};
use tokio::process::Command as TokioCommand;

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

pub fn mv(from: &PathBuf, to: &PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("mv")
            .args(["-f", from, to])
            .spawn()
            .expect(format!("File move failed -> {}", from).as_str());
    }
}

pub fn cp(from: &PathBuf, to: &PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("cp")
            .args(["-f", "-r", from, to])
            .spawn()
            .expect(format!("File copy failed -> {}", from).as_str());
    }
}

pub fn create(path: &PathBuf) {
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

pub fn clip(pathes: &Vec<PathBuf>) -> io::Result<()> {
    let pathes = pathes
        .iter()
        .map(|p| format!("file://{}", p.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    let echo = Command::new("echo")
        .args(["-e", pathes.as_str()])
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    let xclip = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed file copy to clipboard");
    if let Some(mut stdin) = xclip.stdin {
        stdin.write_all(&echo.stdout)?;
    }
    Ok(())
}

pub async fn nvim(path: &PathBuf) -> io::Result<()> {
    TokioCommand::new("nvim")
        .args([path])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await?;
    Ok(())
}

pub fn eog(path: &PathBuf) -> io::Result<()> {
    if let Some(path) = path.to_str() {
        Command::new("eog").args([path]).spawn()?;
    }
    Ok(())
}

pub fn ffprobe_is_video(path: &PathBuf) -> bool {
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

pub fn vlc(path: &PathBuf) -> io::Result<()> {
    let _ = Command::new("vlc").args([path]).output()?;
    Ok(())
}

pub fn file_roller_open(path: &PathBuf) -> io::Result<()> {
    Command::new("file-roller").args([path]).spawn()?;
    Ok(())
}

pub fn extract_from_archive(path: &PathBuf) -> io::Result<()> {
    let mut file = File::open(path)?;
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;

    match &buffer {
        [0x50, 0x4B, 0x03, 0x04] => extract_zip(path)?,
        [0x1F, 0x8B, ..] => extract_tgz(path)?,
        _ => {}
    }
    Ok(())
}

fn extract_zip(path: &PathBuf) -> io::Result<()> {
    Command::new("unzip").args([path]).spawn()?;
    Ok(())
}

fn extract_tgz(path: &PathBuf) -> io::Result<()> {
    Command::new("tar").arg("xzf").arg(path).spawn()?;
    Ok(())
}

pub fn evince(path: &PathBuf) -> io::Result<()> {
    Command::new("evince").args([path]).spawn()?;
    Ok(())
}
