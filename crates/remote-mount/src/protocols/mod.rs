/// Module containing error types used by protocol handlers.
pub mod errors;

/// Module containing the SSHFS protocol handler implementation.
#[cfg(feature = "protocols-sshfs")]
pub mod sshfs;

use anyhow::Result;
use async_trait::async_trait;

/// `ProtocolHandler` is a trait that defines the common interface for various filesystem protocol handlers.
#[async_trait]
pub trait ProtocolHandler<'r> {
    /// Mounts the filesystem using the specified protocol.
    ///
    /// This method is responsible for performing the necessary steps to mount the filesystem according to the protocol's requirements.
    /// It returns the standard output of the mount command if successful, or an error if the mount process fails.
    async fn mount(&mut self) -> Result<String>;

    /// Unmounts the mounted filesystem.
    ///
    /// This method is responsible for performing the necessary steps to unmount the filesystem.
    /// It returns the standard output of the unmount command if successful, or an error if the unmount process fails.
    async fn unmount(&mut self) -> Result<String>;

    /// Checks if the filesystem is currently mounted.
    ///
    /// Returns true if the filesystem is mounted, and false otherwise.
    fn is_mounted(&self) -> bool;

    /// Checks for missing dependencies required for the protocol handler to work.
    ///
    /// Returns an option containing a vector of missing dependency names if any dependencies are missing,
    /// or None if all dependencies are available.
    fn missing_dependencies(&self) -> Option<Vec<String>>;

    /// Returns the protocol associated with this protocol handler.
    ///
    /// Returns one of the enum values defined in the `Protocol` enum.
    fn protocol(&self) -> Protocol;
}

/// Represents different protocols that can be handled by the `ProtocolHandler`.
#[derive(Debug)]
pub enum Protocol {
    #[cfg(feature = "protocols-sshfs")]
    Sshfs,
}
