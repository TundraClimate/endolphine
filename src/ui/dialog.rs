use crate::action::Action;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{Clear, ClearType},
};
use std::io;
use tui_input::{backend::crossterm as backend, Input};

pub struct Dialog {
    pub input: Input,
    pub action: Action,
}

impl From<Action> for Dialog {
    fn from(value: Action) -> Self {
        Dialog {
            action: value,
            input: "".into(),
        }
    }
}

impl Dialog {
    pub fn write_backend<S: AsRef<str>>(&self, text: S) -> io::Result<()> {
        let text = text.as_ref();
        execute!(io::stdout(), MoveTo(1, 40), Print(text))?;
        backend::write(
            &mut io::stdout(),
            self.input.value(),
            self.input.cursor(),
            ((text.len() + 2) as u16, 40),
            30,
        )
    }
}

pub fn log(text: String) -> io::Result<()> {
    execute!(
        io::stdout(),
        MoveTo(1, 40),
        Clear(ClearType::CurrentLine),
        Print(text)
    )?;
    Ok(())
}
