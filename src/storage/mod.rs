mod backends;

use anyhow::Result;
use core::str::FromStr;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait StorageOperations {
    async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>>;
    async fn exists(&self, path: &Path) -> Result<bool>;
}

#[derive(Debug, Clone)]
pub enum StorageBackend {
    #[cfg(feature = "storage-filesystem")]
    Filesystem(Arc<backends::FilesystemStorage>),
    #[cfg(feature = "storage-s3")]
    S3(Arc<backends::S3Storage>),
    #[cfg(feature = "storage-sshfs")]
    SSHFS(Arc<backends::SSHFSStorage>),
}

impl StorageOperations for StorageBackend {
    async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "storage-filesystem")]
            StorageBackend::Filesystem(storage) => storage.read(path).await,
            #[cfg(feature = "storage-s3")]
            StorageBackend::S3(storage) => storage.read(path).await,
            #[cfg(feature = "storage-sshfs")]
            StorageBackend::SSHFS(storage) => storage.read(path).await,
        }
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        match self {
            #[cfg(feature = "storage-filesystem")]
            StorageBackend::Filesystem(storage) => storage.exists(path).await,
            #[cfg(feature = "storage-s3")]
            StorageBackend::S3(storage) => storage.exists(path).await,
            #[cfg(feature = "storage-sshfs")]
            StorageBackend::SSHFS(storage) => storage.exists(path).await,
        }
    }
}

impl FromStr for StorageBackend {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            #[cfg(feature = "storage-filesystem")]
            _ if s.starts_with("fs://") => {
                use faccess::{AccessMode, PathExt};

                let fs_path = PathBuf::from(s.trim_start_matches("fs://").trim());
                let _ = std::fs::create_dir_all(&fs_path);
                if let Err(err) = fs_path.access(AccessMode::WRITE | AccessMode::READ) {
                    return Err(format!(
                        "Path specified cannot be read from or written to by the current user\n\nError: {err}"
                    ));
                }
                Ok(Self::Filesystem(Arc::new(
                    backends::FilesystemStorage::new(fs_path)
                        .map_err(|err| format!("Failed to create filesystem storage: {err:?}"))?,
                )))
            }

            #[cfg(feature = "storage-s3")]
            _ if s.starts_with("s3://") => {
                let bucket = s
                    .trim_start_matches("s3://")
                    .split('/')
                    .next()
                    .ok_or("S3 URL must include bucket: s3://bucket")?;
                if bucket.is_empty() {
                    return Err("S3 bucket name cannot be empty".to_string());
                }
                Ok(Self::S3(Arc::new(
                    backends::S3Storage::new(bucket.to_string())
                        .map_err(|err| format!("failed to create S3 client: {err:?}"))?,
                )))
            }

            #[cfg(feature = "storage-sshfs")]
            _ if s.starts_with("sshfs://") => {
                let mountpoint = s.trim_start_matches("sshfs://").trim().to_string();
                if mountpoint.is_empty() {
                    return Err("SSHFS mountpoint cannot be empty".to_string());
                }
                Ok(Self::SSHFS(Arc::new(
                    backends::SSHFSStorage::new(mountpoint)
                        .map_err(|err| format!("Failed to create SSHFS storage: {err:?}"))?,
                )))
            }

            _ => {
                let mut valid_sources = Vec::new();
                #[cfg(feature = "storage-filesystem")]
                valid_sources.push("'fs://path'");
                #[cfg(feature = "storage-s3")]
                valid_sources.push("'s3://bucket'");
                #[cfg(feature = "storage-sshfs")]
                valid_sources.push("'sshfs://mountpoint'");

                if valid_sources.is_empty() {
                    Err("No storage backends are enabled".to_string())
                } else {
                    Err(format!("Valid sources are: {}", valid_sources.join(", ")))
                }
            }
        }
    }
}
