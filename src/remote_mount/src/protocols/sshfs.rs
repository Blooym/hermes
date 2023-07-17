use super::{
    errors::{MountError, UnmountError},
    ProtocolHandler,
};
use async_trait::async_trait;
use std::path::Path;
use tokio::process;

/// The sshfs binary to use.
const SSHFS_BIN: &str = "sshfs";

/// The umount binary to use.
const UMOUNT_BIN: &str = "fusermount";

/// The shell to use.
const SHELL_BIN: &str = "sh";

/// The dependendencies for this protocol.
pub const DEPENDENCIES: &[&str] = &[SSHFS_BIN, UMOUNT_BIN, SHELL_BIN];

/// A handler for the sshfs protocol.
#[derive(Debug)]
pub struct Sshfs {
    /// Whether the remote filesystem is mounted.
    mounted: bool,

    /// The mountpoint of the remote filesystem.
    mountpoint: String,

    /// The connection string for sshfs.
    connection_string: String,

    /// Options to pass to sshfs.
    options: String,

    /// The password for the sshfs connection.
    password: String,

    /// Additional arguments to pass to sshfs.
    extra_args: String,
}

impl Sshfs {
    /// Create a new instance of Sshfs.
    pub fn new(
        mountpoint: String,
        connection_string: String,
        options: String,
        password: String,
        extra_args: String,
    ) -> Self {
        Self {
            mounted: false,
            mountpoint,
            connection_string,
            options,
            password,
            extra_args,
        }
    }
}

#[async_trait]
impl ProtocolHandler<'_> for Sshfs {
    fn is_mounted(&self) -> bool {
        self.mounted
    }

    fn all_deps_present(&self) -> Result<(), Vec<String>> {
        let mut missing_deps = Vec::new();

        for dep in DEPENDENCIES {
            if which::which(dep).is_err() {
                missing_deps.push(dep.to_string());
            }
        }

        if missing_deps.is_empty() {
            Ok(())
        } else {
            Err(missing_deps)
        }
    }

    async fn mount(&'_ mut self) -> Result<String, MountError> {
        match self.all_deps_present() {
            Ok(_) => {}
            Err(e) => {
                return Err(MountError::MissingDependencies(e.join(", ")));
            }
        }

        if self.is_mounted() {
            return Err(MountError::AlreadyMounted);
        }

        if !Path::new(&self.mountpoint).exists() {
            return Err(MountError::MountFailed(format!(
                "Path {} does not exist",
                &self.mountpoint
            )));
        }

        let options_str = if self.options.is_empty() {
            String::new()
        } else {
            format!("-o password_stdin,{}", self.options)
        };

        let cmd = format!(
            "echo '{}' | {} {} {} {} {}",
            self.password,
            SSHFS_BIN,
            self.connection_string,
            self.mountpoint,
            options_str,
            self.extra_args
        );

        let proc = process::Command::new(SHELL_BIN)
            .arg("-c")
            .arg(cmd)
            .output()
            .await;

        match proc {
            Ok(output) => {
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                if !stderr.is_empty() {
                    return Err(MountError::MountFailed(stderr));
                }

                self.mounted = true;
                Ok(String::from_utf8(output.stdout).unwrap_or_default())
            }
            Err(e) => Err(MountError::MountFailed(e.to_string())),
        }
    }

    async fn unmount(&mut self) -> Result<String, UnmountError> {
        match self.all_deps_present() {
            Ok(_) => {}
            Err(e) => {
                return Err(UnmountError::MissingDependencies(e.join(", ")));
            }
        }

        if !self.is_mounted() {
            return Err(UnmountError::NotMounted);
        }

        let proc = process::Command::new(UMOUNT_BIN)
            .arg("-u")
            .arg(&self.mountpoint)
            .output()
            .await;

        match proc {
            Ok(output) => {
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                if !stderr.is_empty() {
                    return Err(UnmountError::UnmountFailed(stderr));
                }

                self.mounted = false;
                Ok(String::from_utf8(output.stdout).unwrap_or_default())
            }
            Err(e) => Err(UnmountError::UnmountFailed(e.to_string())),
        }
    }
}
