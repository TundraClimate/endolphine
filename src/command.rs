mod create_file;
mod delete_file;
mod exit_app;
mod mapping;
mod menuctl;
mod mv;
mod paste_file;
mod rename_file;
mod reset_view;
mod search;
mod visual;
mod yank_file;

pub use create_file::{AskCreate, CreateFileOrDir};
pub use delete_file::{AskDelete, DeleteFileOrDir, DeleteSelected};
pub use exit_app::ExitApp;
pub use mapping::Remapping;
pub use menuctl::{MenuMove, MenuToggle};
pub use mv::{EnterDirOrEdit, MoveBottom, MoveDown, MoveParent, MoveTop, MoveUp, PageDown, PageUp};
pub use paste_file::{AskPaste, Paste};
pub use rename_file::{AskRename, Rename};
pub use reset_view::ResetView;
pub use search::{Search, SearchNext};
pub use visual::VisualSelect;
pub use yank_file::Yank;

pub trait Command: Send + Sync {
    fn run(&self) -> Result<(), crate::app::Error>;
}

fn parse_prenum() -> Option<usize> {
    let prenum = crate::app::load_buf()
        .into_iter()
        .take_while(crate::key::Key::is_digit)
        .map(|k| k.as_num())
        .collect::<Vec<_>>();
    let mut sum = 0usize;

    for (i, k) in prenum.into_iter().rev().enumerate() {
        sum += (k - 48) as usize * (10usize.pow(i as u32));
    }

    if sum == 0 { None } else { Some(sum) }
}
