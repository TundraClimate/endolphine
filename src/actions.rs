pub enum Action {
    Previous(usize),
    Next(usize),
    Back,
    Open,
    Create,
    Delete,
    Cut,
    Copy,
    Rename,
    Pending,
    Confirm,
    None,
}
