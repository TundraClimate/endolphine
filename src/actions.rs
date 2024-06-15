use std::path::PathBuf;

pub enum Action {
    Previous(usize),
    Next(usize),
    Create(CreateType),
    Delete(PathBuf),
    Cut(PathBuf),
    Copy(PathBuf),
    Rename(PathBuf),
    Pending,
    Confirm,
    None,
}

pub enum CreateType {
    File,
    Directory,
}
