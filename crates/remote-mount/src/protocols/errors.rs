use std::fmt::Display;

/// Represents an error that can occur during mounting a remote filesystem.
#[derive(Debug)]
pub enum MountError {
    /// The filesystem is already mounted and cannot be mounted again.
    AlreadyMounted,

    /// Mounting the filesystem failed with the given error message.
    MountFailed(String),

    /// Mounting the filesystem failed due to the given missing dependency.
    MissingDependency(String),
}

impl Display for MountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountError::AlreadyMounted => write!(f, "Already mounted"),
            MountError::MountFailed(msg) => write!(f, "Mount failed: {}", msg),
            MountError::MissingDependency(msg) => write!(f, "Missing dependency: {}", msg),
        }
    }
}

/// Represents an error that can occur during unmounting a remote filesystem.
#[derive(Debug)]
pub enum UnmountError {
    /// The filesystem is already unmounted and cannot be unmounted again.
    NotMounted,

    /// Unmounting the filesystem failed with the given error message.
    UnmountFailed(String),

    /// Unmount the filesystem failed due to the given missing dependency.
    MissingDependency(String),
}

impl Display for UnmountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnmountError::NotMounted => write!(f, "Not mounted"),
            UnmountError::UnmountFailed(msg) => write!(f, "Unmount failed: {}", msg),
            UnmountError::MissingDependency(msg) => write!(f, "Missing dependency: {}", msg),
        }
    }
}
