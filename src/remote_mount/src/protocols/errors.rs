#[derive(Debug)]
pub enum MountError {
    AlreadyMounted,
    MountFailed(String),
    MissingDependencies(String),
}

#[derive(Debug)]
pub enum UnmountError {
    NotMounted,
    UnmountFailed(String),
    MissingDependencies(String),
}
