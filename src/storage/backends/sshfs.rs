use crate::storage::{FileMetadata, StorageOperations};
use anyhow::{Context, Result, bail};
use std::{io::Write, path::Path, process::Command};
use tokio::io::{self, AsyncRead};
use tracing::debug;

const SSHFS_BIN: &str = "sshfs";
const FUSERMOUNT_BIN: &str = "fusermount";
const DEPENDENCIES: &[&str] = &[SSHFS_BIN, FUSERMOUNT_BIN];

#[derive(Debug)]
pub struct SSHFSStorage {
    mountpoint: Box<Path>,
    connection_string: Box<str>,
    password: Option<Box<str>>,
    options: Option<Box<[String]>>,
}

impl SSHFSStorage {
    pub fn new<M: AsRef<Path>>(mountpoint: M) -> Result<Self> {
        if let Some(missing_deps) = Self::missing_dependencies() {
            bail!(
                "The following dependencies are missing or not in $PATH: {:#?}",
                missing_deps
            );
        };
        let mountpoint = mountpoint.as_ref();
        std::fs::create_dir_all(mountpoint)?;
        let storage = Self {
            mountpoint: std::fs::canonicalize(mountpoint)?.into_boxed_path(),
            connection_string: std::env::var("SSHFS_CONNECTION_STRING")
                .context("SSHFS_CONNECTION_STRING environment variable is required")?
                .into_boxed_str(),
            password: std::env::var("SSHFS_PASSWORD")
                .ok()
                .map(|p| p.into_boxed_str()),
            options: std::env::var("SSHFS_OPTIONS").ok().map(|opts| {
                opts.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            }),
        };
        storage.mount()?;
        Ok(storage)
    }

    fn missing_dependencies() -> Option<Vec<String>> {
        debug!("Checking for missing dependencies from {:?}", DEPENDENCIES);
        let missing_deps = DEPENDENCIES
            .iter()
            .filter_map(|dep| {
                if which::which(dep).is_err() {
                    Some(dep.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if missing_deps.is_empty() {
            None
        } else {
            Some(missing_deps)
        }
    }

    fn mount(&self) -> Result<()> {
        let mut sshfs_cmd = Command::new(SSHFS_BIN);
        sshfs_cmd
            .arg(&*self.connection_string)
            .arg(&*self.mountpoint)
            .arg("-f") // Run in foreground to ensure it's a child of this process
            .arg("-o")
            .arg("ConnectTimeout=10") // Only wait 10 seconds for the connection
            .arg("-o")
            .arg("ro") // Mount as a read-only filesystem
            .arg("-o")
            .arg("ServerAliveInterval=15") // Keep-Alive ping every 15 seconds.
            .arg("-o")
            .arg("reconnect"); // Automatically reconnect on disconnect.

        // Append additional user-provided options.
        if let Some(ref options) = self.options {
            for option in options {
                debug!("Adding user provided option '-o {option}' to args");
                sshfs_cmd.arg("-o").arg(option);
            }
        }
        // Pipe the password provided by the user on mount.
        if self.password.is_some() {
            sshfs_cmd.arg("-o").arg("password_stdin");
            sshfs_cmd.stdin(std::process::Stdio::piped());
        }
        debug!("Mounting SSHFS with args {:?}", sshfs_cmd.get_args());
        let mut child = sshfs_cmd.spawn().context("Failed to spawn sshfs process")?;
        if let Some(password) = &self.password
            && let Some(stdin) = child.stdin.take()
        {
            debug!("Writing SSHFS password to sshfs process stdin");
            let mut stdin = stdin;
            stdin
                .write_all(password.as_bytes())
                .context("Failed to write password to sshfs")?;
            stdin
                .write_all(b"\n")
                .context("Failed to write newline to sshfs")?;
            stdin.flush().context("Failed to close stdin")?;
            debug!("Finished writing SSHFS password");
        }
        debug!("SSHFS mount process completed - sshfs spawned as child process");
        Ok(())
    }
}

impl Drop for SSHFSStorage {
    fn drop(&mut self) {
        debug!("Unmounting SSHFS mountpoint using {FUSERMOUNT_BIN} as backend has been dropped");
        let mountpoint = self.mountpoint.clone();
        let _ = std::process::Command::new(FUSERMOUNT_BIN)
            .arg("-u")
            .arg(&*mountpoint)
            .output();
    }
}

impl StorageOperations for SSHFSStorage {
    async fn read_stream(&self, path: &Path) -> Result<Option<Box<dyn AsyncRead + Unpin + Send>>> {
        let path = Path::new(&*self.mountpoint).join(path);
        debug!("Reading file stream {path:?}");
        match tokio::fs::File::open(&path).await {
            Ok(file) => Ok(Some(Box::new(file))),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn metadata(&self, path: &Path) -> Result<Option<FileMetadata>> {
        let path = Path::new(&*self.mountpoint).join(path);
        debug!("Reading file metadata at {path:?}");
        match tokio::fs::metadata(&path).await {
            Ok(metadata) => Ok(Some(FileMetadata {
                file_size: metadata.len().try_into()?,
            })),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}
