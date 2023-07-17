pub mod errors;
#[cfg(feature = "protocol-sshfs")]
pub mod sshfs;

use async_trait::async_trait;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocols {
    #[cfg(feature = "protocol-sshfs")]
    Sshfs,
}

impl FromStr for Protocols {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "protocol-sshfs")]
            "sshfs" => Ok(Self::Sshfs),
            _ => Err(format!("Invalid protocol: {}", s)),
        }
    }
}

/// A trait for handling a filesystem protocol / mounting filesystems.
#[async_trait]
pub trait ProtocolHandler<'r> {
    /// Mount the remote filesystem.
    async fn mount(&mut self) -> Result<String, errors::MountError>;

    /// Unmount the remote filesystem.
    async fn unmount(&mut self) -> Result<String, errors::UnmountError>;

    /// Check to see if the remote filesystem is mounted.
    fn is_mounted(&self) -> bool;

    /// Check to see if the required dependencies for this protocol are installed.
    /// Returns an error with a vector of missing dependencies if any are missing.
    fn all_deps_present(&self) -> Result<(), Vec<String>>;
}
