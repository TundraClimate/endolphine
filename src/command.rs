mod create_file;
mod delete_file;
mod exit_app;
mod menuctl;
mod mv;
mod paste_file;
mod rename_file;
mod reset_view;
mod search;
mod visual;
mod yank_file;

pub use create_file::AskCreate;
pub use delete_file::AskDelete;
pub use exit_app::ExitApp;
pub use menuctl::{MenuMove, MenuToggle};
pub use mv::{EnterDirOrEdit, Move, MoveParent};
pub use paste_file::AskPaste;
pub use rename_file::AskRename;
pub use reset_view::ResetView;
pub use search::Search;
pub use visual::VisualSelect;
pub use yank_file::Yank;

pub trait Command {
    fn run(&self) -> Result<(), crate::app::Error>;
}
