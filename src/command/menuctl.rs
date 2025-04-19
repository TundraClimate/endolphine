use super::Command;
use crate::{canvas, menu};

pub struct MenuToggle;

impl Command for MenuToggle {
    fn run(&self) -> Result<(), crate::Error> {
        if !menu::is_opened() || menu::refs().is_enabled() {
            menu::refs().toggle_enable();
        }

        menu::toggle_open();
        canvas::cache_clear();

        Ok(())
    }
}

pub struct MenuMove;

impl Command for MenuMove {
    fn run(&self) -> Result<(), crate::Error> {
        if !menu::is_opened() {
            menu::toggle_open();
        }

        menu::refs().toggle_enable();
        canvas::cache_clear();

        Ok(())
    }
}
