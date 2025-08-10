use std::io;

const LOCAL_CLIPBOARD: &str = "/tmp/endolphine/cb.txt";

pub fn command() -> &'static str {
    match WindowSystem::load() {
        WindowSystem::Macos => "clipboard",
        WindowSystem::Wayland => "wl-copy, wl-paste",
        WindowSystem::X11 => "xclip",
    }
}

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

pub fn is_cmd_installed() -> bool {
    match WindowSystem::load() {
        WindowSystem::Macos => is_macos_cmd_installed(),
        WindowSystem::Wayland => is_wayland_cmd_installed(),
        WindowSystem::X11 => is_x11_cmd_installed(),
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

    let script = format!(
        r#"""
        ObjC.import("CoreServices");
        ObjC.import("AppKit");
        function mimeToUTI(m) {{
            return $.UTTypeCreatePreferredIdentifierForTag(
                $.kUTTagClassMIMEType,
                m,
                null
            ).takeRetainedValue().toString();
        }}
        let pb = $.NSPasteboard.generalPasteboard;
        let str = pb.stringForType(mimeToUTI("{}"));
        if (str) {{
            console.log(ObjC.unwrap(str));
        }} else {{
            console.log("");
        }}
        """#,
        ty
    );

    let output = Command::new("osascript")
        .args(["-l", "JavaScript", "-e"])
        .arg(script)
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
    use std::process::{Command, Stdio};

    let script = format!(
        r#"""
        ObjC.import("CoreServices");
        ObjC.import("AppKit");
        function mimeToUTI(m) {{
          return $.UTTypeCreatePreferredIdentifierForTag($.kUTTagClassMIMEType, m, null)
            .takeRetainedValue()
            .toString();
        }}
        let pb = $.NSPasteboard.generalPasteboard;
        pb.clearContents;
        let str = $(ObjC.unwrap("{}"));
        pb.setStringForType(str, mimeToUTI("{}"));
        """#,
        s.as_ref(),
        ty
    );

    Command::new("osascript")
        .args(["-l", "JavaScript", "-e"])
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

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

fn is_macos_cmd_installed() -> bool {
    true
}

fn is_wayland_cmd_installed() -> bool {
    use std::process::{Command, Stdio};

    Command::new("wl-copy")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
        && Command::new("wl-paste")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
}

fn is_x11_cmd_installed() -> bool {
    use std::process::{Command, Stdio};

    Command::new("xclip")
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
