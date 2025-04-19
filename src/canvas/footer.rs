use super::Widget;
use crate::{app, theme};
use crossterm::{
    cursor::MoveTo,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};

pub struct Footer;

impl Widget for Footer {
    const ID: u8 = 2;

    fn render(size: (u16, u16)) -> Result<(), crate::Error> {
        let procs = app::procs();
        let bar_text = format!(
            "{}{} {} process running",
            SetBackgroundColor(theme::bar_color()),
            SetForegroundColor(theme::scheme().bar_text),
            procs
        );

        Footer::cached_render_row(
            &procs.to_string(),
            size.1 - 2,
            format!(
                "{}{}{}",
                Print(super::colored_bar(theme::bar_color(), size.0)),
                MoveTo(super::get_view_shift(), size.1 - 2),
                Print(bar_text),
            ),
        )?;

        Ok(())
    }
}
