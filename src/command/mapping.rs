use super::Command;

pub struct Remapping(pub crate::key::Keymap);

impl Command for Remapping {
    fn run(&self) -> Result<(), crate::app::Error> {
        crate::app::sync_key_buf(self.0.clone());
        if let Some(cmd_res) =
            crate::config::eval_keymap(crate::app::current_mode(), &crate::app::load_buf())
        {
            cmd_res?
        }

        Ok(())
    }
}
