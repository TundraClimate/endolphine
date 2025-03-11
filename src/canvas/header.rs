use super::Widget;
use crate::{app, cursor, error::*, misc, theme};
use crossterm::{
    cursor::MoveTo,
    style::{Print, SetBackgroundColor, SetForegroundColor},
};

pub struct Header;

impl Widget for Header {
    const ID: u8 = 0;

    fn render(size: (u16, u16)) -> EpResult<()> {
        let current_path = app::get_path();
        let filename = format!("{}/", misc::file_name(&current_path));

        let usr = option_env!("USER").map_or("/root".to_string(), |u| match u {
            "root" => "/root".to_string(),
            user => format!("/home/{}", user),
        });

        let parent =
            misc::parent(&current_path)
                .to_str()
                .map_or("_OsCompatible_".to_string(), |p| {
                    let rep = p.replacen(&usr, "~", 1);
                    match rep.as_str() {
                        "/" => "".to_string(),
                        replaced => format!("{}/", replaced),
                    }
                });

        let pwd = format!(
            "{}{}{}",
            parent,
            SetForegroundColor(theme::scheme().path_picked),
            filename
        );

        Header::cached_render_row(
            &filename.to_string(),
            0,
            format!(" {} in {}", filename, pwd),
        )?;

        let cursor = cursor::load();

        let page = cursor.current() / misc::body_height() as usize + 1;
        let len = misc::child_files_len(&app::get_path());

        let page_area = format!(
            "{}{} Page {} {}(All {} items)",
            SetBackgroundColor(theme::bar_color()),
            SetForegroundColor(theme::scheme().bar_text),
            page,
            SetForegroundColor(theme::scheme().bar_text_light),
            len
        );

        Header::cached_render_row(
            &format!("{}{}", page, len),
            1,
            format!(
                "{}{}{}",
                Print(super::colored_bar(theme::bar_color(), size.0)),
                MoveTo(super::get_view_shift(), 1),
                Print(page_area)
            ),
        )?;

        Ok(())
    }
}
