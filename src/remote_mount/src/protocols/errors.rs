#[derive(Debug)]
pub enum ProtocolError {
    MissingConfigurationOption(String),
    InvalidConfigurationOption(String),
}

#[derive(Debug)]
pub enum MountError {
    AlreadyMounted,
    MountFailed(String),
}

#[derive(Debug)]
pub enum UnmountError {
    NotMounted,
    UnmountFailed(String),
}
