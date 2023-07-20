use crate::env::{errors::EnvCreateError, FromEnv};
use remote_mount::protocols::sshfs::Sshfs;
use std::env;

/// The options for the sshfs protocol.
#[derive(Debug)]
pub struct SshfsOptions {
    pub mountpoint: String,
    pub connection_string: String,
    pub options: String,
    pub password: String,
    pub extra_args: String,
}

impl SshfsOptions {
    /// Create a new instance of SshfsOptions.
    pub fn new(
        mountpoint: String,
        connection_string: String,
        options: String,
        password: String,
        extra_args: String,
    ) -> Self {
        Self {
            mountpoint,
            connection_string,
            options,
            password,
            extra_args,
        }
    }

    /// Create a new instance of Sshfs from the options provided.
    pub fn create_handler_from_opts(&self) -> Sshfs {
        Sshfs::new(
            self.mountpoint.clone(),
            self.connection_string.clone(),
            self.options.clone(),
            self.password.clone(),
            self.extra_args.clone(),
        )
    }
}

const VAR_MOUNTPOINT: &str = "HERMES_SSHFS_MOUNTPOINT";
const VAR_CONNECTION_STRING: &str = "HERMES_SSHFS_CONNECTION_STRING";
const VAR_PASSWORD: &str = "HERMES_SSHFS_PASSWORD";
const VAR_OPTIONS: &str = "HERMES_SSHFS_OPTIONS";
const VAR_EXTRA_ARGS: &str = "HERMES_SSHFS_ARGS";

impl FromEnv for SshfsOptions {
    fn from_env() -> Result<Self, EnvCreateError> {
        Ok(Self::new(
            env::var(VAR_MOUNTPOINT)
                .map_err(|_| EnvCreateError::MissingVariable(VAR_MOUNTPOINT.into()))?,
            env::var(VAR_CONNECTION_STRING)
                .map_err(|_| EnvCreateError::MissingVariable(VAR_CONNECTION_STRING.into()))?,
            env::var(VAR_OPTIONS).unwrap_or_default(),
            env::var(VAR_PASSWORD)
                .map_err(|_| EnvCreateError::MissingVariable(VAR_PASSWORD.into()))?,
            env::var(VAR_EXTRA_ARGS).unwrap_or_default(),
        ))
    }
}
