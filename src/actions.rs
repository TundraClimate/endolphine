pub enum Action {
    Previous(usize),
    Next(usize),
    Create,
    Delete,
    Cut,
    Copy,
    Rename,
    Pending,
    Confirm,
    None,
}
