pub mod errors;
#[cfg(feature = "protocols-sshfs")]
pub mod sshfs;

use self::errors::{MountError, UnmountError};
use async_trait::async_trait;

#[derive(Debug)]
pub enum Protocols {
    #[cfg(feature = "protocols-sshfs")]
    Sshfs,
}

/// A handler for a remote filesystem protocol.
#[async_trait]
pub trait ProtocolHandler<'r> {
    /// Mount the remote filesystem.
    ///
    /// Returns a success message or a mount error.
    async fn mount(&mut self) -> Result<String, MountError>;

    /// Unmount the remote filesystem.
    ///
    /// Returns a success message or an unmount error.
    async fn unmount(&mut self) -> Result<String, UnmountError>;

    /// Returns whether the remote filesystem is mounted.
    fn is_mounted(&self) -> bool;

    /// Returns a list of missing dependencies that are required for this protocol to work
    /// or None if there are no missing dependencies.
    fn missing_dependencies(&self) -> Option<Vec<String>>;

    /// Returns the protocol that this handler implements.
    fn protocol(&self) -> Protocols;
}
