use super::{
    errors::{MountError, UnmountError},
    ProtocolHandler, Protocols,
};
use async_trait::async_trait;
use std::path::Path;
use tokio::process::Command;

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
    async fn mount(&'_ mut self) -> Result<String, MountError> {
        match self.missing_dependencies() {
            Some(missing_deps) => {
                return Err(MountError::MissingDependencies(missing_deps.join(", ")));
            }
            None => {}
        }

        // Make sure we aren't already mounted.
        if self.is_mounted() {
            return Err(MountError::AlreadyMounted);
        }

        // Ensure the mountpoint exists.
        if !Path::new(&self.mountpoint).exists() {
            return Err(MountError::MountFailed(format!(
                "Path {} does not exist",
                &self.mountpoint
            )));
        }

        // Find the sshfs binary.
        let sshfs_location =
            which::which(SSHFS_BIN).map_err(|e| MountError::MissingDependencies(e.to_string()))?;
        let shell_location =
            which::which(SHELL_BIN).map_err(|e| MountError::MissingDependencies(e.to_string()))?;

        // Create the options string.
        let options_str = if self.options.is_empty() {
            String::new()
        } else {
            format!("-o password_stdin,{}", self.options)
        };

        // Create the command.
        let cmd = format!(
            "echo '{}' | {} {} {} {} {}",
            self.password,
            sshfs_location.display(),
            self.connection_string,
            self.mountpoint,
            options_str,
            self.extra_args
        );

        // Run the command.
        let proc = Command::new(shell_location)
            .arg("-c")
            .arg(cmd)
            .output()
            .await;

        // Return the result.
        match proc {
            Ok(output) => {
                // Check for errors in stderr.
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                if !stderr.is_empty() {
                    return Err(MountError::MountFailed(stderr));
                }

                // If there were no errors, we're mounted.
                self.mounted = true;
                Ok(String::from_utf8(output.stdout).unwrap_or_default())
            }
            Err(e) => Err(MountError::MountFailed(e.to_string())),
        }
    }

    async fn unmount(&mut self) -> Result<String, UnmountError> {
        // Ensure we aren't missing any dependencies.
        match self.missing_dependencies() {
            Some(missing_deps) => {
                return Err(UnmountError::MissingDependencies(missing_deps.join(", ")));
            }
            None => {}
        }

        // Ensure we're mounted.
        if !self.is_mounted() {
            return Err(UnmountError::NotMounted);
        }

        // Find the umount binary.
        let umount_location = which::which(UMOUNT_BIN)
            .map_err(|e| UnmountError::MissingDependencies(e.to_string()))?;

        // Run the command.
        let proc = Command::new(umount_location)
            .arg("-u")
            .arg(&self.mountpoint)
            .output()
            .await;

        // Return the result.
        match proc {
            Ok(output) => {
                // Check for errors in stderr.
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                if !stderr.is_empty() {
                    return Err(UnmountError::UnmountFailed(stderr));
                }

                // If there were no errors, we're unmounted.
                self.mounted = false;
                Ok(String::from_utf8(output.stdout).unwrap_or_default())
            }
            Err(e) => Err(UnmountError::UnmountFailed(e.to_string())),
        }
    }

    fn is_mounted(&self) -> bool {
        self.mounted
    }

    fn missing_dependencies(&self) -> Option<Vec<String>> {
        let mut missing_deps = Vec::new();

        for dep in DEPENDENCIES {
            if which::which(dep).is_err() {
                missing_deps.push(dep.to_string());
            }
        }

        if missing_deps.is_empty() {
            None
        } else {
            Some(missing_deps)
        }
    }

    fn protocol(&self) -> Protocols {
        Protocols::Sshfs
    }
}
