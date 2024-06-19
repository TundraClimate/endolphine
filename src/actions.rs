use std::path::PathBuf;

pub enum Action {
    Previous(usize),
    Next(usize),
    Create,
    Delete,
    Cut,
    Copy,
    Rename(PathBuf),
    Pending,
    Confirm,
    None,
}
