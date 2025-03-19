use super::Widget;
use crate::{app, menu, misc, theme};
use crossterm::{
    cursor::MoveTo,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
};

pub struct Menu;

fn render_menu_line(
    row: u16,
    tag: &str,
    slide_len: u16,
    is_cursor_pos: bool,
    menu_enabled: bool,
) -> Result<(), app::Error> {
    let tag = tag.chars().take(slide_len as usize - 6).collect::<String>();
    let cur = if is_cursor_pos { ">" } else { " " };
    let under_name_color = SetBackgroundColor(theme::widget_item_bg(is_cursor_pos, menu_enabled));

    Menu::cached_render_row(
        &format!("{}{}", cur, tag),
        row,
        format!(
            "{} |{} {}{} {}{}",
            cur,
            under_name_color,
            SetForegroundColor(theme::scheme().menu_tag),
            tag,
            SetBackgroundColor(theme::widget_bg()),
            ResetColor,
        ),
    )?;
    Ok(())
}

impl Widget for Menu {
    const ID: u8 = 3;

    fn render(_size: (u16, u16)) -> Result<(), app::Error> {
        let slide_len = super::get_view_shift();
        if slide_len == 0 {
            return Ok(());
        }

        Menu::cached_render_row(
            &format!("{:?}", theme::scheme().label),
            0,
            format!(
                "{} Select to Cd {}",
                SetBackgroundColor(theme::scheme().label),
                ResetColor
            ),
        )?;
        Menu::cached_render_row(
            &format!("{:?}", theme::wid_bar_color()),
            1,
            super::colored_bar(theme::wid_bar_color(), slide_len - 1),
        )?;

        let menu = menu::refs();
        let cursor = &menu.cursor;

        for i in 2..misc::body_height() + 3 {
            if let Some(element) = menu.elements.get(i as usize - 2) {
                let is_cursor_pos = i as usize - 2 == cursor.current();

                render_menu_line(i, &element.tag, slide_len, is_cursor_pos, menu.is_enabled())?;
            } else {
                Menu::cached_render_row("empty", i, "".to_string())?;
            }
        }

        Ok(())
    }

    fn render_row(row: u16, cmds: String) -> std::io::Result<()> {
        let slide = super::get_view_shift();
        let fg = theme::widget_fg();
        let bg = theme::widget_bg();
        crossterm::queue!(
            std::io::stdout(),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
            MoveTo(0, row),
            Print(" ".repeat(slide as usize)),
            MoveTo(0, row),
            Print(cmds),
            MoveTo(slide - 1, row),
            SetBackgroundColor(theme::wid_bar_color()),
            SetForegroundColor(theme::scheme().label),
            Print("|"),
            SetForegroundColor(fg),
            SetBackgroundColor(bg),
        )
    }
}
