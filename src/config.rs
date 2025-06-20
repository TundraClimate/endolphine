use std::{
    io,
    path::{Path, PathBuf},
};

pub fn file_path() -> Option<PathBuf> {
    option_env!("HOME").map(|home| {
        Path::new(home)
            .join(".config")
            .join("endolphine")
            .join("config.toml")
    })
}

pub fn setup_local() -> io::Result<()> {
    use std::fs;

    let Some(config_path) = file_path() else {
        panic!("Couldn't read the $HOME");
    };

    if !config_path.exists() {
        let parent = config_path.parent().unwrap();

        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, b"")?;
    }

    Ok(())
}
