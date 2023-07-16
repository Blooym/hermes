use std::env;

#[derive(Debug)]
pub enum ProtocolError {
    MissingConfigurationOption(String),
}

/// A trait for creating a new instance of a protocol handler from environment variables and a specified mountpoint.
pub trait FromEnv {
    /// Create a new instance from environment variables.
    fn from_env() -> Result<Self, ProtocolError>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct SshfsOptions {
    pub mountpoint: String,
    pub connection_string: String,
    pub options: String,
    pub password: String,
    pub extra_args: String,
}

impl FromEnv for SshfsOptions {
    fn from_env() -> Result<Self, ProtocolError> {
        let mountpoint = env::var("HERMES_SSHFS_MOUNTPOINT").map_err(|_| {
            ProtocolError::MissingConfigurationOption("HERMES_SSHFS_MOUNTPOINT".into())
        })?;

        let connection_string = env::var("HERMES_SSHFS_CONNECTION_STRING").map_err(|_| {
            ProtocolError::MissingConfigurationOption("HERMES_SSHFS_CONNECTION_STRING".into())
        })?;

        let password = env::var("HERMES_SSHFS_PASSWORD").map_err(|_| {
            ProtocolError::MissingConfigurationOption("HERMES_SSHFS_PASSWORD".into())
        })?;

        let options = env::var("HERMES_SSHFS_OPTIONS").unwrap_or_default();
        let extra_args = env::var("HERMES_SSHFS_ARGS").unwrap_or_default();

        Ok(Self {
            mountpoint,
            connection_string,
            options,
            password,
            extra_args,
        })
    }
}
