pub mod branch;
pub mod cli;
pub mod error;
pub mod repository;
pub mod resolver;
pub mod staging;
pub mod commit;

#[allow(unused_imports)]
pub use cli::GitCli;
pub use error::GitError;
pub use repository::{Git2Repository, GitRepository};

#[allow(unused_imports)]
pub use resolver::{GitResolver, GitSource, GitVersion, ResolvedGit};

#[allow(unused_imports)]
pub use repository::{RefInfo, RefType};
#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::repository::FileChangeKind;
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
            let initial_path = dir.path().join("initial.txt");
            std::fs::write(&initial_path, "initial content\n").expect("Failed to write initial.txt");
            index
                .add_path(Path::new("README.md"))
                .expect("Failed to add to index");
            index
                .add_path(Path::new("initial.txt"))
                .expect("Failed to add initial.txt to index");
            index.write().expect("Failed to write index");
            index.write_tree().expect("Failed to write tree")
        };
        let tree = repo.find_tree(tree_id).expect("Failed to find tree");
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .expect("Failed to create initial commit");
        drop(tree);

        (dir, repo)
    }

    /// Create a temporary git repository with an initial commit and a local bare remote.
    /// The initial commit is pushed to the bare remote with upstream tracking configured.
    /// Returns (working_dir, bare_dir) — caller must keep both TempDirs alive.
    pub fn create_test_repo_with_remote() -> (tempfile::TempDir, tempfile::TempDir) {
        // Create bare remote
        let bare_dir = tempfile::TempDir::new().expect("Failed to create bare temp dir");
        git2::Repository::init_bare(bare_dir.path()).expect("Failed to init bare repo");

        // Create working repo with initial commit
        let (work_dir, work_repo) = create_test_repo();

        // Add bare as "origin" remote
        work_repo
            .remote("origin", bare_dir.path().to_str().unwrap())
            .expect("Failed to add remote");

        // Push initial commit to bare remote via CLI
        let work_path = work_dir.path().to_str().unwrap();
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(work_path)
            .args(["push", "-u", "origin", "HEAD"])
            .output()
            .expect("Failed to push to bare repo");
        assert!(
            output.status.success(),
            "Failed to push initial commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        (work_dir, bare_dir)
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

        let cli = GitCli::new(repo_path, "git");
        cli.create_branch("feature-1", None).unwrap();

        let branch = repo.find_branch("feature-1", git2::BranchType::Local);
        assert!(branch.is_ok());
    }

    #[test]
    fn test_switch_branch() {
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        let cli = GitCli::new(repo_path, "git");
        cli.create_branch("feature-1", None).unwrap();
        cli.switch_branch("feature-1").unwrap();

        let head = repo.head().unwrap();
        assert_eq!(head.shorthand(), Some("feature-1"));
    }

    #[test]
    fn test_create_duplicate_branch_fails() {
        let (dir, _repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        let cli = GitCli::new(repo_path, "git");
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
    fn test_changed_files_modified_file() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::write(dir.path().join("initial.txt"), "modified content").unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "initial.txt");
        assert_eq!(files[0].staged, None);
        assert_eq!(files[0].unstaged, Some(FileChangeKind::Modified));
    }

    #[test]
    fn test_changed_files_new_untracked_file() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::write(dir.path().join("new.txt"), "new content").unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "new.txt");
        assert_eq!(files[0].staged, None);
        assert_eq!(files[0].unstaged, Some(FileChangeKind::Added));
    }

    #[test]
    fn test_changed_files_staged_only() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::write(dir.path().join("staged.txt"), "staged content").unwrap();
        let bare = git2::Repository::open(dir.path()).unwrap();
        let mut index = bare.index().unwrap();
        index.add_path(std::path::Path::new("staged.txt")).unwrap();
        index.write().unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "staged.txt");
        assert_eq!(files[0].staged, Some(FileChangeKind::Added));
        assert_eq!(files[0].unstaged, None);
    }

    #[test]
    fn test_changed_files_partially_staged() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::write(dir.path().join("initial.txt"), "staged version").unwrap();
        let bare = git2::Repository::open(dir.path()).unwrap();
        let mut index = bare.index().unwrap();
        index.add_path(std::path::Path::new("initial.txt")).unwrap();
        index.write().unwrap();
        std::fs::write(dir.path().join("initial.txt"), "unstaged version").unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "initial.txt");
        assert_eq!(files[0].staged, Some(FileChangeKind::Modified));
        assert_eq!(files[0].unstaged, Some(FileChangeKind::Modified));
    }

    #[test]
    fn test_changed_files_deleted() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::remove_file(dir.path().join("initial.txt")).unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "initial.txt");
        assert_eq!(files[0].staged, None);
        assert_eq!(files[0].unstaged, Some(FileChangeKind::Deleted));
    }

    #[test]
    fn test_changed_files_clean_repo() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        let files = repo.changed_files().unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_changed_files_invalid_path() {
        let repo = Git2Repository::open("/nonexistent/path");
        assert!(repo.changed_files().is_err());
    }

    #[test]
    fn test_changed_files_sorted_by_path() {
        let (dir, _) = create_test_repo();
        let repo = Git2Repository::open(dir.path());
        std::fs::write(dir.path().join("z-file.txt"), "z").unwrap();
        std::fs::write(dir.path().join("a-file.txt"), "a").unwrap();
        std::fs::write(dir.path().join("m-file.txt"), "m").unwrap();
        let files = repo.changed_files().unwrap();
        assert_eq!(files.len(), 3);
        assert_eq!(files[0].path, "a-file.txt");
        assert_eq!(files[1].path, "m-file.txt");
        assert_eq!(files[2].path, "z-file.txt");
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

        let cli = GitCli::new(path, "git");
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

    /// Integration test: resolve git binary, construct GitCli with resolved path,
    /// create a branch, and verify it exists via Git2Repository.
    #[test]
    fn test_integration_resolve_and_create_branch() {
        // Step 1: Resolve git binary
        let resolved = GitResolver::resolve().expect("git should be resolvable on dev machine");
        assert!(
            resolved.path.exists(),
            "resolved git path should exist: {:?}",
            resolved.path
        );
        assert!(
            resolved.version.major > 2
                || (resolved.version.major == 2 && resolved.version.minor >= 35),
            "resolved git version should be >= 2.35, got {}",
            resolved.version
        );

        // Step 2: Create temp repo with initial commit
        let (dir, repo) = create_test_repo();
        let repo_path = dir.path().to_str().unwrap();

        // Step 3: Construct GitCli with the resolved path
        let cli = GitCli::new(repo_path, &resolved.path);

        // Step 4: Create a branch via the CLI
        cli.create_branch("integration-test-branch", None)
            .expect("branch creation should succeed with resolved git");

        // Step 5: Verify the branch exists via Git2Repository (read path)
        let branch = repo
            .find_branch("integration-test-branch", git2::BranchType::Local)
            .expect("branch should exist after creation");
        assert!(
            branch.name().unwrap().is_some(),
            "branch should have a name"
        );

        // Step 6: Verify source is either EnvOverride or SystemPath
        assert!(
            matches!(
                resolved.source,
                resolver::GitSource::EnvOverride | resolver::GitSource::SystemPath
            ),
            "source should be EnvOverride or SystemPath, got {:?}",
            resolved.source
        );
    }
}
