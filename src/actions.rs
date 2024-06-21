pub enum Action {
    Previous(usize),
    Next(usize),
    Back,
    Open,
    Create,
    Delete,
    Cut,
    Copy,
    Paste,
    Rename,
    Pending,
    PreConfirm,
    Confirm,
    Clean,
    None,
}
