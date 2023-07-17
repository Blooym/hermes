pub mod errors;

use self::errors::EnvCreateError;

/// A trait for creating an instance from environment variables.
pub trait FromEnv {
    /// Create a new instance from environment variables.
    fn from_env() -> Result<Self, EnvCreateError>
    where
        Self: Sized;
}
