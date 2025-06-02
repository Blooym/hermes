use crate::storage::StorageOperations;
use anyhow::Result;
use std::{
    fs::{self},
    io::{self},
    path::{Component, Path, PathBuf},
};
use tracing::debug;

#[derive(Debug)]
pub struct FilesystemStorage {
    base_path: std::path::PathBuf,
}

impl FilesystemStorage {
    pub fn new(base_path: PathBuf) -> Result<Self> {
        let _ = fs::create_dir_all(&base_path);
        Ok(Self {
            base_path: fs::canonicalize(base_path)?,
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
    async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let path = self.base_path.join(path);
        debug!("Reading file at {path:?}");
        match fs::read(path) {
            Ok(str) => Ok(Some(str)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        let path = self.join_to_base(path)?;
        debug!("Checking if a file exists at {path:?}");
        Ok(fs::exists(path)?)
    }
}
