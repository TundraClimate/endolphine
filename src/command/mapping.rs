use super::Command;

pub struct Remapping(pub crate::key::Keymap);

impl Command for Remapping {
    fn run(&self) -> Result<(), crate::app::Error> {
        let keymap = self.0.as_vec();
        let (mut begin, mut end) = (0usize, 0usize);

        keymap.iter().enumerate().try_for_each(|(i, key)| {
            end = i + 1;

            if key.is_digit() {
                return Ok(());
            }

            let keymap = &keymap[begin..end];

            if crate::config::has_map(keymap, crate::app::current_mode()) {
                crate::app::sync_key_buf(crate::key::Keymap::new(keymap));
                if let Some(cmd_res) =
                    crate::config::eval_keymap(crate::app::current_mode(), keymap)
                {
                    cmd_res?;
                }

                begin = end;
            }

            Ok(())
        })
    }
}
