use crate::{ui, Args};
use std::{error::Error, path::PathBuf};

pub struct App {
    opened_path: PathBuf,
    current_path: PathBuf,
}

impl App {
    pub fn new(args: Args) -> App {
        App {
            opened_path: args.path.clone(),
            current_path: args.path.clone(),
        }
    }

    pub fn init(self) -> Result<App, Box<dyn Error>> {
        Ok(self)
    }

    pub fn run_app(self) -> Result<(), Box<dyn Error>> {
        ui::render_mode()?;
        Ok(())
    }
}
