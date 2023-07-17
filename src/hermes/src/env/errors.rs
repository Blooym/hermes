/// An error that can occur when creating an instance from environment variables.
#[derive(Debug)]
pub enum EnvCreateError {
    /// A required environment variable is missing.
    MissingVariable(String),
}
