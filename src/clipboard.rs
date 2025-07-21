use std::io;

const LOCAL_CLIPBOARD: &str = "/tmp/endolphine/cb.txt";

pub fn read() -> io::Result<String> {
    use std::{fs, path::Path};

    let path = Path::new(LOCAL_CLIPBOARD);

    fs::read_to_string(path)
}

pub fn read_native(ty: &str) -> io::Result<String> {
    let wsys = WindowSystem::load();

    match wsys {
        WindowSystem::Macos => read_macos(ty),
        WindowSystem::Wayland => read_wayland(ty),
        WindowSystem::X11 => read_x11(ty),
    }
}

pub fn clip<S: AsRef<str>>(s: S) -> io::Result<()> {
    use std::{fs, path::Path};

    let path = Path::new(LOCAL_CLIPBOARD);

    fs::write(path, s.as_ref())
}

pub fn clip_native<S: AsRef<str>>(s: S, ty: &str) -> io::Result<()> {
    let wsys = WindowSystem::load();

    match wsys {
        WindowSystem::Macos => clip_macos(s, ty),
        WindowSystem::Wayland => clip_wayland(s, ty),
        WindowSystem::X11 => clip_x11(s, ty),
    }
}

enum WindowSystem {
    Macos,
    Wayland,
    X11,
}

impl WindowSystem {
    fn load() -> Self {
        use std::env::consts::OS;

        if OS == "macos" {
            WindowSystem::Macos
        } else if option_env!("WAYLAND_DISPLAY").is_some() {
            WindowSystem::Wayland
        } else {
            WindowSystem::X11
        }
    }
}

fn read_macos(ty: &str) -> io::Result<String> {
    use std::process::{Command, Stdio};

    let output = Command::new("clipboard")
        .args(["-pboard", "general", "-type", ty])
        .stderr(Stdio::null())
        .output()?;
    let output = String::from_utf8_lossy(output.stdout.as_slice());

    Ok(String::from(output))
}

fn read_wayland(ty: &str) -> io::Result<String> {
    use std::process::{Command, Stdio};

    let output = Command::new("wl-paste")
        .args(["-t", ty])
        .stderr(Stdio::null())
        .output()?;
    let output = String::from_utf8_lossy(output.stdout.as_slice());

    Ok(String::from(output))
}

fn read_x11(ty: &str) -> io::Result<String> {
    use std::process::{Command, Stdio};

    let output = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", ty, "-o"])
        .stderr(Stdio::null())
        .output()?;
    let output = String::from_utf8_lossy(output.stdout.as_slice());

    Ok(String::from(output))
}

fn clip_macos<S: AsRef<str>>(s: S, ty: &str) -> io::Result<()> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    let cmd = Command::new("clipboard")
        .args(["-pboard", "general", "-type", ty])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = cmd.stdin {
        write!(&mut stdin, "{}", s.as_ref())?;
    }

    Ok(())
}

fn clip_wayland<S: AsRef<str>>(s: S, ty: &str) -> io::Result<()> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    let cmd = Command::new("wl-copy")
        .args(["-t", ty])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = cmd.stdin {
        write!(&mut stdin, "{}", s.as_ref())?;
    }

    Ok(())
}

fn clip_x11<S: AsRef<str>>(s: S, ty: &str) -> io::Result<()> {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };

    let cmd = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", ty])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = cmd.stdin {
        write!(&mut stdin, "{}", s.as_ref())?;
    }

    Ok(())
}
