//! Error types for thongna operations

use std::fmt;

/// Error type for thongna operations
#[derive(Debug)]
pub enum Error {
    /// Dictionary not found
    DictionaryNotFound(String),
    /// Lock acquisition failed
    LockError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::DictionaryNotFound(name) => {
                write!(f, "Dictionary name {} does not exist.", name)
            }
            Error::LockError(msg) => {
                write!(f, "Failed to acquire dictionary lock: {}", msg)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type alias for thongna operations
pub type Result<T> = std::result::Result<T, Error>;
