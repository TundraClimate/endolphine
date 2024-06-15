use clap::Parser;
use endolphine::{app::App, Args};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let app = App::new(args);
    app.init()?.run_app()?;

    Ok(())
}
