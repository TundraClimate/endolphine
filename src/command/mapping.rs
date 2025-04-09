use super::Command;

pub struct Remapping(pub crate::app::AppMode, pub crate::key::Keymap);

impl Command for Remapping {
    fn run(&self) -> Result<(), crate::app::Error> {
        crate::app::sync_key_buf(self.1.clone());
        if let Some(cmd_res) = crate::config::eval_keymap(self.0, &crate::app::load_buf()) {
            cmd_res?
        }

        Ok(())
    }
}
