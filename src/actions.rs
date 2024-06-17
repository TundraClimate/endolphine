use std::path::PathBuf;

pub enum Action {
    Previous(usize),
    Next(usize),
    Create(CreateType),
    Delete(PathBuf),
    Cut,
    Copy,
    Rename(PathBuf),
    Pending,
    Confirm,
    None,
}

pub enum CreateType {
    File,
    Directory,
}
