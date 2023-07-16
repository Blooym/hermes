use async_trait::async_trait;
use std::str::FromStr;

pub mod errors;
#[cfg(feature = "protocol-sshfs")]
pub mod sshfs;

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
    /// Mount the filesystem.
    async fn mount(&mut self) -> Result<String, errors::MountError>;

    /// Unmount the filesystem.
    async fn unmount(&mut self) -> Result<String, errors::UnmountError>;

    /// Returns true if the remote filesystem is mounted.
    fn is_mounted(&self) -> bool;
}
