use crate::storage::StorageOperations;
use anyhow::Result;
use std::path::{Component, Path, PathBuf};
use tokio::io::{self, AsyncRead};
use tracing::debug;

#[derive(Debug)]
pub struct FilesystemStorage {
    base_path: std::path::PathBuf,
}

impl FilesystemStorage {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let _ = std::fs::create_dir_all(&base_path);
        Ok(Self {
            base_path: std::fs::canonicalize(base_path)?,
        })
    }
}

impl FilesystemStorage {
    fn join_to_base(&self, path: &Path) -> Result<PathBuf> {
        for component in path.components() {
            match component {
                Component::Prefix(_) | Component::RootDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("Absolute paths are not allowed: {:?}", path),
                    )
                    .into());
                }
                Component::ParentDir => {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        format!("Paths cannot reference a parent directory: {:?}", path),
                    )
                    .into());
                }
                _ => {}
            }
        }
        Ok(self.base_path.join(path))
    }
}

impl StorageOperations for FilesystemStorage {
    async fn read_stream(&self, path: &Path) -> Result<Option<Box<dyn AsyncRead + Unpin + Send>>> {
        let path = self.base_path.join(path);
        debug!("Reading file at {path:?}");
        match tokio::fs::File::open(&path).await {
            Ok(file) => Ok(Some(Box::new(file))),
            Err(err) if err.kind() == tokio::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        let path = self.join_to_base(path)?;
        debug!("Checking if a file exists at {path:?}");
        Ok(tokio::fs::try_exists(path).await?)
    }
}
