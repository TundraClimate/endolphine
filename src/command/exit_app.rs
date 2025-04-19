use super::Command;

pub struct ExitApp;

impl Command for ExitApp {
    fn run(&self) -> Result<(), crate::Error> {
        exit()
    }
}

fn exit() -> Result<(), crate::Error> {
    crate::app::disable_tui()?;

    crate::sys_log!("i", "Endolphine close successfully");

    std::process::exit(0)
}
