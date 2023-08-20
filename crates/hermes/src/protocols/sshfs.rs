use crate::traits::from_env::{FromEnv, FromEnvError};
use remote_mount::protocols::sshfs::Sshfs;
use std::env;

const VAR_MOUNTPOINT: &str = "HERMES_SSHFS_MOUNTPOINT";
const VAR_CONNECTION_STRING: &str = "HERMES_SSHFS_CONNECTION_STRING";
const VAR_PASSWORD: &str = "HERMES_SSHFS_PASSWORD";
const VAR_OPTIONS: &str = "HERMES_SSHFS_OPTIONS";
const VAR_EXTRA_ARGS: &str = "HERMES_SSHFS_ARGS";

#[derive(Debug)]
pub struct SshfsOptions {
    pub mountpoint: String,
    pub connection_string: String,
    pub options: String,
    pub password: String,
    pub extra_args: String,
}

impl SshfsOptions {
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

    pub fn create_handler_from_self(&self) -> Sshfs {
        Sshfs::new(
            self.mountpoint.clone(),
            self.connection_string.clone(),
            self.options.clone(),
            self.password.clone(),
            self.extra_args.clone(),
        )
    }
}

impl FromEnv for SshfsOptions {
    fn from_env() -> Result<Self, FromEnvError> {
        Ok(Self::new(
            env::var(VAR_MOUNTPOINT)
                .map_err(|_| FromEnvError::MissingVariable(VAR_MOUNTPOINT.into()))?,
            env::var(VAR_CONNECTION_STRING)
                .map_err(|_| FromEnvError::MissingVariable(VAR_CONNECTION_STRING.into()))?,
            env::var(VAR_OPTIONS).unwrap_or_default(),
            env::var(VAR_PASSWORD)
                .map_err(|_| FromEnvError::MissingVariable(VAR_PASSWORD.into()))?,
            env::var(VAR_EXTRA_ARGS).unwrap_or_default(),
        ))
    }
}
