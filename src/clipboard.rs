use std::{
    io::Write,
    process::{Command, Stdio},
};

enum WindowSystem {
    MacOS,
    Wayland,
    X11,
}

fn window_system() -> WindowSystem {
    if std::env::consts::OS == "macos" {
        WindowSystem::MacOS
    } else if option_env!("WAYLAND_DISPLAY").is_some() {
        WindowSystem::Wayland
    } else {
        WindowSystem::X11
    }
}

const MACOS_CB_CMD: &str = "clipboard";
const WAYLAND_CB_CMD: [&str; 2] = ["wl-copy", "wl-paste"];
const X11_CB_CMD: &str = "xclip";

pub fn is_cmd_installed() -> bool {
    WindowSystem::is_cmd_installed()
}

pub fn clip(text: &str) {
    *APP_REGISTER.write().unwrap() = text.to_string();
}

pub fn clip_native(text: &str, ty: &str) -> std::io::Result<()> {
    WindowSystem::clip(text, ty)
}

pub fn read_clipboard() -> String {
    APP_REGISTER.read().unwrap().to_string()
}

pub fn read_clipboard_native(ty: &str) -> std::io::Result<String> {
    WindowSystem::read_clipboard(ty)
}

impl WindowSystem {
    fn is_cmd_installed() -> bool {
        match window_system() {
            WindowSystem::MacOS => Self::macos_cbcmd_installed(),
            WindowSystem::Wayland => Self::wayland_cbcmd_installed(),
            WindowSystem::X11 => Self::x11_cbcmd_installed(),
        }
    }

    fn macos_cbcmd_installed() -> bool {
        Command::new(MACOS_CB_CMD)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    fn wayland_cbcmd_installed() -> bool {
        Command::new(WAYLAND_CB_CMD[0])
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
            && Command::new(WAYLAND_CB_CMD[1])
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
    }

    fn x11_cbcmd_installed() -> bool {
        Command::new(X11_CB_CMD)
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    }

    fn clip(text: &str, ty: &str) -> std::io::Result<()> {
        match window_system() {
            Self::MacOS => Self::macos_clip(text, ty),
            WindowSystem::Wayland => Self::wayland_clip(text, ty),
            WindowSystem::X11 => Self::x11_clip(text, ty),
        }
    }

    fn macos_clip(text: &str, ty: &str) -> std::io::Result<()> {
        let cmd = Command::new(MACOS_CB_CMD)
            .args(["-pboard", "general", "-type", ty])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = cmd.stdin {
            write!(&mut stdin, "{}", text)?;
        }

        Ok(())
    }

    fn wayland_clip(text: &str, ty: &str) -> std::io::Result<()> {
        let cmd = Command::new(WAYLAND_CB_CMD[0])
            .args(["-t", ty])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = cmd.stdin {
            write!(&mut stdin, "{}", text)?;
        }

        Ok(())
    }

    fn x11_clip(text: &str, ty: &str) -> std::io::Result<()> {
        let cmd = Command::new(X11_CB_CMD)
            .args(["-selection", "clipboard", "-t", ty])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = cmd.stdin {
            write!(&mut stdin, "{}", text)?;
        }

        Ok(())
    }

    fn read_clipboard(ty: &str) -> std::io::Result<String> {
        match window_system() {
            WindowSystem::MacOS => Self::macos_read(ty),
            WindowSystem::Wayland => Self::wayland_read(ty),
            WindowSystem::X11 => Self::x11_read(ty),
        }
    }

    fn macos_read(ty: &str) -> std::io::Result<String> {
        let output = Command::new(MACOS_CB_CMD)
            .args(["-pboard", "general", "-type", ty])
            .stderr(Stdio::null())
            .output()?;
        let output = String::from_utf8_lossy(output.stdout.as_slice());
        Ok(String::from(output))
    }

    fn wayland_read(ty: &str) -> std::io::Result<String> {
        let output = Command::new(WAYLAND_CB_CMD[1])
            .args(["-t", ty])
            .stderr(Stdio::null())
            .output()?;
        let output = String::from_utf8_lossy(output.stdout.as_slice());
        Ok(String::from(output))
    }

    fn x11_read(ty: &str) -> std::io::Result<String> {
        let output = Command::new(X11_CB_CMD)
            .args(["-selection", "clipboard", "-t", ty, "-o"])
            .stderr(Stdio::null())
            .output()?;
        let output = String::from_utf8_lossy(output.stdout.as_slice());
        Ok(String::from(output))
    }
}

crate::global! {
    static APP_REGISTER: std::sync::RwLock<String> = std::sync::RwLock::new(String::new());
}
