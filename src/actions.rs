use std::path::PathBuf;

pub enum Action {
    Previous(i32),
    Next(i32),
    Create(CreateType),
    Delete(PathBuf),
    Move(PathBuf),
    Rename(PathBuf),
    None,
}

pub enum CreateType {
    File,
    Directory,
}
