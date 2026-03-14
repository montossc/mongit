/// Unified error type for all git operations in mongit.
///
/// Covers errors from git2 (read path), CLI invocations (write path),
/// repository discovery, and I/O operations.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// Error from the git2 library (read operations)
    #[error("git2: {0}")]
    Git2(#[from] git2::Error),

    /// Error from git CLI execution (write operations)
    #[error("git cli: {0}")]
    Cli(String),

    /// Repository or ref not found
    #[error("not found: {0}")]
    NotFound(String),

    /// Filesystem I/O error
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Convert GitError to String for Tauri IPC serialization.
/// Tauri commands return `Result<T, String>`, so this bridge is required.
impl From<GitError> for String {
    fn from(err: GitError) -> Self {
        err.to_string()
    }
}
