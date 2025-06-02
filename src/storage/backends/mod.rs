#[cfg(not(any(
    feature = "storage-filesystem",
    feature = "storage-s3",
    feature = "storage-sshfs"
)))]
compile_error!("At least one storage backend must be enabled");

#[cfg(feature = "storage-filesystem")]
mod filesystem;
#[cfg(feature = "storage-filesystem")]
pub use filesystem::FilesystemStorage;
#[cfg(feature = "storage-s3")]
mod s3;
#[cfg(feature = "storage-s3")]
pub use s3::S3Storage;
#[cfg(feature = "storage-sshfs")]
mod sshfs;
#[cfg(feature = "storage-sshfs")]
pub use sshfs::SSHFSStorage;
