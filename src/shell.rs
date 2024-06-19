use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
};

pub fn trash_file(path: PathBuf) {
    let home = dirs::home_dir().unwrap();
    if let (Some(path), Some(home)) = (path.to_str(), home.to_str()) {
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

pub fn mv(from: PathBuf, to: PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("mv")
            .args(["-f", from, to])
            .spawn()
            .expect(format!("File move failed -> {}", from).as_str());
    }
}

pub fn cp(from: PathBuf, to: PathBuf) {
    if let (Some(from), Some(to)) = (from.to_str(), to.to_str()) {
        Command::new("cp")
            .args(["-f", "-r", from, to])
            .spawn()
            .expect(format!("File copy failed -> {}", from).as_str());
    }
}

pub fn create(path: PathBuf) {
    if let Some(path) = path.to_str() {
        Command::new("touch")
            .args([path])
            .spawn()
            .expect(format!("File create failed -> {}", path).as_str());
    }
}

pub fn mkdir(path: PathBuf) {
    if let Some(path) = path.to_str() {
        Command::new("mkdir")
            .args([path])
            .spawn()
            .expect(format!("Directory create failed -> {}", path).as_str());
    }
}

pub fn clip(pathes: Vec<PathBuf>) {
    let pathes = pathes
        .iter()
        .map(|p| format!("file://{}", p.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    let echo = Command::new("echo")
        .args(["-e", pathes.as_str()])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    let xclip = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    if let Some(mut stdin) = xclip.stdin {
        stdin.write_all(&echo.stdout).unwrap();
    }
}
