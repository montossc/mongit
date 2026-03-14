pub mod cli;
pub mod error;
pub mod repository;

#[allow(unused_imports)]
pub use cli::GitCli;
pub use error::GitError;
pub use repository::{Git2Repository, GitRepository};

#[allow(unused_imports)]
pub use repository::{RefInfo, RefType};
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
        let err = GitError::CommandFailed {
            cmd: "status".into(),
            stderr: "fatal: not a git repository".into(),
            exit_code: Some(128),
        };
        let s: String = err.into();
        assert!(s.contains("git cli failed"));
        assert!(s.contains("not a git repository"));
    }

    #[test]
    fn test_create_branch() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        let cli = GitCli::new(repo_path);
        cli.create_branch("feature-1", None).unwrap();

        let branch = repo.find_branch("feature-1", git2::BranchType::Local);
        assert!(branch.is_ok());
    }

    #[test]
    fn test_switch_branch() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        let cli = GitCli::new(repo_path);
        cli.create_branch("feature-1", None).unwrap();
        cli.switch_branch("feature-1").unwrap();

        let head = repo.head().unwrap();
        assert_eq!(head.shorthand(), Some("feature-1"));
    }

    #[test]
    fn test_create_duplicate_branch_fails() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        let cli = GitCli::new(repo_path);
        cli.create_branch("feature-1", None).unwrap();
        let err = cli.create_branch("feature-1", None).unwrap_err();

        assert!(matches!(err, GitError::CommandFailed { .. }));
    }

    #[test]
    fn test_status_clean_repo() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let repo = Git2Repository::open(path);
        let status = repo.status().expect("status should work");
        assert_eq!(status.changed_files, 0);
        assert_eq!(status.staged_files, 0);
    }

    #[test]
    fn test_status_with_changes() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        std::fs::write(dir.path().join("new_file.txt"), "hello\n").expect("write should succeed");

        let repo = Git2Repository::open(path);
        let status = repo.status().expect("status should work");
        assert_eq!(status.changed_files, 1);
    }

    #[test]
    fn test_log_returns_commits() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let repo = Git2Repository::open(path);
        let commits = repo.log(10).expect("log should work");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_log_all_branches() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let cli = GitCli::new(path);
        cli.create_branch("feature-1", None).unwrap();

        let repo = Git2Repository::open(path);
        let commits = repo.log_all_branches(100).expect("log_all should work");
        assert!(!commits.is_empty());
        assert_eq!(commits[0].message, "Initial commit");
    }

    #[test]
    fn test_branches_lists_default() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let repo = Git2Repository::open(path);
        let branches = repo.branches().expect("branches should work");
        assert!(!branches.is_empty());
    }

    #[test]
    fn test_refs() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let repo = Git2Repository::open(path);
        let refs = repo.refs().expect("refs should work");
        assert!(refs.len() >= 2);
        assert!(refs
            .iter()
            .any(|r| matches!(r.ref_type, repository::RefType::Head)));
        assert!(refs
            .iter()
            .any(|r| matches!(r.ref_type, repository::RefType::LocalBranch)));
    }

    #[test]
    fn test_current_branch() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        let repo = Git2Repository::open(path);
        let branch = repo.current_branch().expect("current_branch should work");
        assert!(matches!(branch.as_deref(), Some("main") | Some("master")));
    }

    #[test]
    fn test_diff_workdir_shows_changes() {
        let (dir, _repo) = create_test_repo();
        let path = dir.path().to_str().expect("path should be utf-8");

        std::fs::write(dir.path().join("diff_file.txt"), "line 1\nline 2\n")
            .expect("write should succeed");

        let repo = Git2Repository::open(path);
        let diff = repo.diff_workdir().expect("diff should work");
        assert!(!diff.is_empty());
        assert!(diff
            .iter()
            .any(|entry| entry.path.ends_with("diff_file.txt")));
    }
}

    #[test]
    fn test_perf_log_all_branches_1k() {
        use std::time::Instant;

        let dir = tempfile::TempDir::new().expect("temp dir");
        let repo = git2::Repository::init(dir.path()).expect("init");

        // Set up initial commit
        let sig = repo.signature().unwrap_or_else(|_| {
            git2::Signature::now("Test", "test@test.com").unwrap()
        });
        let tree_id = {
            let mut index = repo.index().unwrap();
            let path = dir.path().join("README.md");
            std::fs::write(&path, "# test").unwrap();
            index.add_path(std::path::Path::new("README.md")).unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        let mut last_oid = repo
            .commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        // Create 1000 commits
        let commit_count = 1000;
        for i in 1..commit_count {
            let path = dir.path().join("README.md");
            std::fs::write(&path, format!("# commit {i}")).unwrap();
            let mut index = repo.index().unwrap();
            index.add_path(std::path::Path::new("README.md")).unwrap();
            let new_tree_id = index.write_tree().unwrap();
            let new_tree = repo.find_tree(new_tree_id).unwrap();
            let parent = repo.find_commit(last_oid).unwrap();
            last_oid = repo
                .commit(Some("HEAD"), &sig, &sig, &format!("Commit {i}"), &new_tree, &[&parent])
                .unwrap();
        }

        let path_str = dir.path().to_str().unwrap();

        let repo = Git2Repository::open(path_str);

        // Time the revwalk
        let start = Instant::now();
        let commits = repo.log_all_branches(10000).expect("log should work");
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        eprintln!(
            "PERF: log_all_branches({} commits) = {:.1}ms ({:.1}μs/commit)",
            commits.len(),
            elapsed_ms,
            elapsed_ms * 1000.0 / commits.len() as f64
        );

        assert_eq!(commits.len(), commit_count);
        // Must complete under 500ms for 1k commits (generous bound)
        assert!(
            elapsed_ms < 500.0,
            "log_all_branches too slow: {elapsed_ms:.1}ms for {commit_count} commits"
        );
    }
