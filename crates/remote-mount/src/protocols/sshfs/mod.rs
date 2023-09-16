use super::{
    errors::{MountError, UnmountError},
    Protocol, ProtocolHandler,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use log::{debug, info};
use std::process::Output;
use tokio::process::Command;

const SSHFS_BIN: &str = "sshfs";
const UMOUNT_BIN: &str = "fusermount";
const SHELL_BIN: &str = "sh";

pub const DEPENDENCIES: &[&str] = &[SSHFS_BIN, UMOUNT_BIN, SHELL_BIN];

#[derive(Debug)]
pub struct Sshfs {
    mounted: bool,
    mountpoint: String,
    connection_string: String,
    options: String,
    password: String,
    extra_args: String,
}

impl Sshfs {
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

    async fn execute_command(&self, shell: &str, command: &str) -> Result<Output> {
        info!(
            "Executing command: {} -c {}",
            shell,
            command.replace(&self.password, "**********")
        );

        let cmd = format!("{} -c '{}'", shell, command);

        let proc = Command::new(shell)
            .args(["-c", &cmd])
            .output()
            .await
            .context("Process returned non-zero exit code")?;
        debug!("Command output: {:?}", proc);

        Ok(proc)
    }
}

#[async_trait]
impl ProtocolHandler<'_> for Sshfs {
    async fn mount(&mut self) -> Result<String> {
        info!("Mounting filesystem at {}", self.mountpoint);

        if let Some(missing_deps) = self.missing_dependencies() {
            anyhow::bail!(
                "Unable to unmount filesystem, the following dependencies are missing or not in $PATH: {:#?}",
                missing_deps
            );
        }

        if self.is_mounted() {
            anyhow::bail!(MountError::AlreadyMounted);
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

        let proc = self.execute_command(SHELL_BIN, &cmd).await?;

        let stderr = String::from_utf8_lossy(&proc.stderr);
        if !stderr.is_empty() {
            anyhow::bail!(MountError::MountFailed(stderr.to_string()));
        }

        self.mounted = true;
        info!("Successfully mounted filesystem at {}", self.mountpoint);
        Ok(String::from_utf8_lossy(&proc.stdout).to_string())
    }

    async fn unmount(&mut self) -> Result<String> {
        info!("Unmounting filesystem at {}", self.mountpoint);

        if let Some(missing_deps) = self.missing_dependencies() {
            anyhow::bail!(
                "Unable to unmount filesystem, the following dependencies are missing or not in $PATH: {:#?}",
                missing_deps
            );
        }

        if !self.is_mounted() {
            anyhow::bail!(UnmountError::NotMounted);
        }

        let proc = self
            .execute_command(SHELL_BIN, &format!("{} -u {}", UMOUNT_BIN, self.mountpoint))
            .await?;

        let stderr = String::from_utf8_lossy(&proc.stderr);
        if !stderr.is_empty() {
            anyhow::bail!(UnmountError::UnmountFailed(stderr.to_string()));
        }

        self.mounted = false;
        info!("Successfully unmounted filesystem at {}", self.mountpoint);
        Ok(String::from_utf8_lossy(&proc.stdout).to_string())
    }

    fn is_mounted(&self) -> bool {
        self.mounted
    }

    fn missing_dependencies(&self) -> Option<Vec<String>> {
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

    fn protocol(&self) -> Protocol {
        Protocol::Sshfs
    }
}
