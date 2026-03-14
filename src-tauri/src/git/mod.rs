pub mod error;

pub use error::GitError;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Create a temporary git repository with an initial commit.
    /// Returns (TempDir, Repository) — caller must keep TempDir alive.
    pub fn create_test_repo() -> (tempfile::TempDir, git2::Repository) {
        let dir = tempfile::TempDir::new().expect("Failed to create temp dir");
        let repo = git2::Repository::init(dir.path()).expect("Failed to init repo");

        // Configure test user
        let mut config = repo.config().expect("Failed to get config");
        config
            .set_str("user.name", "Test User")
            .expect("Failed to set user.name");
        config
            .set_str("user.email", "test@example.com")
            .expect("Failed to set user.email");

        // Create initial commit
        let sig = git2::Signature::now("Test User", "test@example.com")
            .expect("Failed to create signature");
        let tree_id = {
            let mut index = repo.index().expect("Failed to get index");

            // Write a test file so the tree isn't empty
            let file_path = dir.path().join("README.md");
            std::fs::write(&file_path, "# Test repo\n").expect("Failed to write README");
            index
                .add_path(Path::new("README.md"))
                .expect("Failed to add to index");
            index.write().expect("Failed to write index");
            index.write_tree().expect("Failed to write tree")
        };
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .expect("Failed to create initial commit");
        drop(tree);

        (dir, repo)
    }

    #[test]
    fn test_helper_creates_valid_repo() {
        let (_dir, repo) = create_test_repo();
        assert!(!repo.is_empty().unwrap());
        assert!(repo.head().is_ok());
        let head = repo.head().unwrap();
        assert!(head.shorthand().is_some());
    }

    #[test]
    fn test_git_error_from_string() {
        let err = GitError::NotFound("branch main".into());
        let s: String = err.into();
        assert!(s.contains("not found"));
        assert!(s.contains("branch main"));
    }

    #[test]
    fn test_git_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: GitError = io_err.into();
        let s: String = err.into();
        assert!(s.contains("io:"));
    }

    #[test]
    fn test_git_error_cli() {
        let err = GitError::Cli("fatal: not a git repository".into());
        let s: String = err.into();
        assert!(s.contains("git cli:"));
    }
}
