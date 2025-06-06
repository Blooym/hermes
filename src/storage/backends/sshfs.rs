use crate::storage::StorageOperations;
use anyhow::{Context, Result, bail};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use tracing::debug;

const SSHFS_BIN: &str = "sshfs";
const FUSERMOUNT_BIN: &str = "fusermount";
pub const DEPENDENCIES: &[&str] = &[SSHFS_BIN, FUSERMOUNT_BIN];

#[derive(Debug)]
pub struct SSHFSStorage {
    mountpoint: PathBuf,
    connection_string: String,
    password: Option<String>,
    options: Option<Vec<String>>,
}

impl SSHFSStorage {
    pub fn new<S: Into<PathBuf>>(mountpoint: S) -> Result<Self> {
        if let Some(missing_deps) = Self::missing_dependencies() {
            bail!(
                "The following dependencies are missing or not in $PATH: {:#?}",
                missing_deps
            );
        };
        let mountpoint = mountpoint.into();
        fs::create_dir_all(&mountpoint)?;
        let storage = Self {
            mountpoint: fs::canonicalize(mountpoint)?,
            connection_string: std::env::var("SSHFS_CONNECTION_STRING")
                .context("SSHFS_CONNECTION_STRING environment variable is required")?,
            password: std::env::var("SSHFS_PASSWORD").ok(),
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
            .arg(&self.connection_string)
            .arg(&self.mountpoint)
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
        if let Some(password) = &self.password {
            if let Some(stdin) = child.stdin.take() {
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
            .arg(&mountpoint)
            .output();
    }
}

impl StorageOperations for SSHFSStorage {
    async fn read(&self, path: &Path) -> Result<Option<Vec<u8>>> {
        let path = Path::new(&self.mountpoint).join(path);
        debug!("Reading file at {path:?}");
        match fs::read(path) {
            Ok(str) => Ok(Some(str)),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    async fn exists(&self, path: &Path) -> Result<bool> {
        let path = Path::new(&self.mountpoint).join(path);
        debug!("Checking if a file exists at {path:?}");
        Ok(fs::exists(&path)?)
    }
}
