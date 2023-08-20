/// An error that can occur when creating an instance from environment variables.
#[derive(Debug)]
pub enum FromEnvError {
    /// A required environment variable is missing.
    MissingVariable(String),
}

/// A trait for creating an instance from environment variables.
pub trait FromEnv {
    /// Create a new instance from environment variables.
    fn from_env() -> Result<Self, FromEnvError>
    where
        Self: Sized;
}
