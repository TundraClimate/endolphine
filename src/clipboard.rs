use std::{
    io::Write,
    process::{Command, Stdio},
};

enum WindowSystem {
    Wayland,
    X11,
}

fn window_system() -> WindowSystem {
    if option_env!("WAYLAND_DISPLAY").is_some() {
        WindowSystem::Wayland
    } else {
        WindowSystem::X11
    }
}

const WAYLAND_CB_CMD: &str = "wl-copy";
const X11_CB_CMD: &str = "xclip";

pub fn is_cmd_installed() -> bool {
    WindowSystem::is_cmd_installed()
}

pub fn clip(text: &str, ty: &str) -> std::io::Result<()> {
    WindowSystem::clip(text, ty)
}

impl WindowSystem {
    fn is_cmd_installed() -> bool {
        match window_system() {
            WindowSystem::Wayland => Self::wayland_cbcmd_installed(),
            WindowSystem::X11 => Self::x11_cbcmd_installed(),
        }
    }

    fn wayland_cbcmd_installed() -> bool {
        Command::new(WAYLAND_CB_CMD)
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
            WindowSystem::Wayland => Self::wayland_clip(text, ty),
            WindowSystem::X11 => Self::x11_clip(text, ty),
        }
    }

    fn wayland_clip(text: &str, ty: &str) -> std::io::Result<()> {
        let cmd = Command::new(WAYLAND_CB_CMD)
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
}
