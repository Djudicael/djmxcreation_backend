use std::fmt;

/// Errors that can occur when loading application configuration from the environment.
#[derive(Debug)]
pub enum ConfigError {
    /// A required environment variable is missing.
    Missing(&'static str),
    /// An environment variable has an invalid value.
    Invalid { key: &'static str, reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(key) => write!(f, "required environment variable '{key}' is not set"),
            Self::Invalid { key, reason } => {
                write!(f, "environment variable '{key}' is invalid: {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}
