use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::Serialize;

use super::GitError;

#[derive(Debug, Clone, Serialize)]
pub struct MergeState {
    pub is_merging: bool,
    pub incoming_sha: Option<String>,
    pub conflicted_files: Vec<ConflictFileEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictFileEntry {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictContent {
    pub file_path: String,
    pub base: Option<String>,
    pub ours: String,
    pub theirs: String,
}

pub fn get_merge_state(path: &Path) -> Result<MergeState, GitError> {
    let repo = git2::Repository::open(path)?;
    let merge_head_path = repo.path().join("MERGE_HEAD");
    let is_merging = merge_head_path.exists();

    let incoming_sha = if is_merging {
        Some(std::fs::read_to_string(merge_head_path)?.trim().to_string())
    } else {
        None
    };

    let mut conflicted_paths = HashSet::new();
    let index = repo.index()?;
    if index.has_conflicts() {
        for conflict in index.conflicts()? {
            let conflict = conflict?;
            if let Some(path) = conflict_path(&conflict) {
                conflicted_paths.insert(path);
            }
        }
    }

    let mut conflicted_files: Vec<ConflictFileEntry> = conflicted_paths
        .into_iter()
        .map(|path| ConflictFileEntry { path })
        .collect();
    conflicted_files.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(MergeState {
        is_merging,
        incoming_sha,
        conflicted_files,
    })
}

pub fn get_conflict_content(path: &Path, file_path: &str) -> Result<ConflictContent, GitError> {
    let repo = git2::Repository::open(path)?;
    let index = repo.index()?;

    if !index.has_conflicts() {
        return Err(GitError::NotFound(
            "repository has no conflicted files".to_string(),
        ));
    }

    for conflict in index.conflicts()? {
        let conflict = conflict?;

        if conflict_path(&conflict).as_deref() != Some(file_path) {
            continue;
        }

        let base = read_stage_content(&repo, conflict.ancestor.as_ref())?;
        let ours = read_stage_content(&repo, conflict.our.as_ref())?.unwrap_or_default();
        let theirs = read_stage_content(&repo, conflict.their.as_ref())?.unwrap_or_default();

        return Ok(ConflictContent {
            file_path: file_path.to_string(),
            base,
            ours,
            theirs,
        });
    }

    Err(GitError::NotFound(format!(
        "file not found in conflicted set: {file_path}"
    )))
}

/// Write resolved content to the working tree file and stage it in the git index.
/// This removes the file from the conflict state.
pub fn resolve_conflict(
    path: &Path,
    file_path: &str,
    content: &str,
) -> Result<(), GitError> {
    let repo = git2::Repository::open(path)?;

    // Write resolved content to the working tree
    let abs_file = path.join(file_path);
    fs::write(&abs_file, content)?;

    // Stage the file in the index (removes conflict entries for this path)
    let mut index = repo.index()?;
    index.add_path(Path::new(file_path))?;
    index.write()?;

    Ok(())
}

/// Abort the current merge: remove merge state files and reset working tree to HEAD.
pub fn abort_merge(path: &Path) -> Result<(), GitError> {
    let repo = git2::Repository::open(path)?;

    // Clean up merge state (removes MERGE_HEAD, MERGE_MSG, MERGE_MODE)
    repo.cleanup_state()?;

    // Reset working tree to HEAD
    let head = repo.head()?.peel_to_commit()?;
    let head_tree = head.tree()?;
    repo.checkout_tree(
        head_tree.as_object(),
        Some(
            git2::build::CheckoutBuilder::new()
                .force()
                .remove_untracked(false),
        ),
    )?;

    // Reset index to HEAD
    repo.reset(head.as_object(), git2::ResetType::Mixed, None)?;

    Ok(())
}

/// Complete the merge by creating a merge commit with both parent SHAs.
/// Uses MERGE_MSG as the default message if none is provided.
pub fn complete_merge(path: &Path, message: Option<&str>) -> Result<String, GitError> {
    let repo = git2::Repository::open(path)?;

    // Verify we're in a merge state
    let merge_head_path = repo.path().join("MERGE_HEAD");
    if !merge_head_path.exists() {
        return Err(GitError::InvalidArgument(
            "no merge in progress".to_string(),
        ));
    }

    // Verify no remaining conflicts
    let index = repo.index()?;
    if index.has_conflicts() {
        return Err(GitError::InvalidArgument(
            "cannot complete merge: unresolved conflicts remain".to_string(),
        ));
    }

    // Get the merge message
    let merge_msg_path = repo.path().join("MERGE_MSG");
    let default_msg = if merge_msg_path.exists() {
        fs::read_to_string(&merge_msg_path)?
    } else {
        "Merge commit".to_string()
    };
    let commit_message = message.unwrap_or(default_msg.trim());

    // Get parent commits: HEAD + MERGE_HEAD
    let head_commit = repo.head()?.peel_to_commit()?;
    let merge_head_sha = fs::read_to_string(&merge_head_path)?.trim().to_string();
    let merge_head_oid = git2::Oid::from_str(&merge_head_sha)?;
    let merge_head_commit = repo.find_commit(merge_head_oid)?;

    // Write the tree from the current index
    let mut index = repo.index()?;
    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;

    // Get signature
    let sig = repo.signature()?;

    // Create merge commit with both parents
    let commit_oid = repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        commit_message,
        &tree,
        &[&head_commit, &merge_head_commit],
    )?;

    // Clean up merge state
    repo.cleanup_state()?;

    Ok(commit_oid.to_string())
}

fn conflict_path(conflict: &git2::IndexConflict) -> Option<String> {
    let entry = conflict
        .ancestor
        .as_ref()
        .or(conflict.our.as_ref())
        .or(conflict.their.as_ref())?;

    Some(String::from_utf8_lossy(&entry.path).to_string())
}

fn read_stage_content(
    repo: &git2::Repository,
    entry: Option<&git2::IndexEntry>,
) -> Result<Option<String>, GitError> {
    let Some(entry) = entry else {
        return Ok(None);
    };

    let blob = repo.find_blob(entry.id)?;
    Ok(Some(String::from_utf8_lossy(blob.content()).to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::create_test_repo;
    use std::path::Path;
    use std::process::Command;

    fn run_git(path: &Path, args: &[&str]) -> std::process::Output {
        Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .expect("git command should run")
    }

    fn default_branch(path: &Path) -> String {
        let output = run_git(path, &["rev-parse", "--abbrev-ref", "HEAD"]);
        assert!(
            output.status.success(),
            "failed to detect branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    fn create_merge_conflict_repo() -> tempfile::TempDir {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        let base_branch = default_branch(path);

        let output = run_git(path, &["checkout", "-b", "feature"]);
        assert!(
            output.status.success(),
            "failed to create feature branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::write(path.join("initial.txt"), "feature content\n")
            .expect("write feature content");
        assert!(run_git(path, &["add", "initial.txt"]).status.success());
        let output = run_git(path, &["commit", "-m", "feature change"]);
        assert!(
            output.status.success(),
            "failed feature commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = run_git(path, &["checkout", &base_branch]);
        assert!(
            output.status.success(),
            "failed checkout base branch: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        std::fs::write(path.join("initial.txt"), "main content\n").expect("write main content");
        assert!(run_git(path, &["add", "initial.txt"]).status.success());
        let output = run_git(path, &["commit", "-m", "main change"]);
        assert!(
            output.status.success(),
            "failed main commit: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let output = run_git(path, &["merge", "feature"]);
        assert!(
            !output.status.success(),
            "merge unexpectedly succeeded without conflict"
        );

        dir
    }

    fn create_merge_no_conflict_no_commit_repo() -> tempfile::TempDir {
        let (dir, _repo) = create_test_repo();
        let path = dir.path();

        let base_branch = default_branch(path);

        let output = run_git(path, &["checkout", "-b", "feature"]);
        assert!(output.status.success());

        std::fs::write(path.join("feature-only.txt"), "feature file\n")
            .expect("write feature file");
        assert!(run_git(path, &["add", "feature-only.txt"]).status.success());
        assert!(run_git(path, &["commit", "-m", "add feature-only file"])
            .status
            .success());

        let output = run_git(path, &["checkout", &base_branch]);
        assert!(output.status.success());

        std::fs::write(path.join("main-only.txt"), "main file\n").expect("write main file");
        assert!(run_git(path, &["add", "main-only.txt"]).status.success());
        assert!(run_git(path, &["commit", "-m", "add main-only file"])
            .status
            .success());

        let output = run_git(path, &["merge", "--no-commit", "feature"]);
        assert!(
            output.status.success(),
            "merge --no-commit should succeed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        assert!(path.join(".git/MERGE_HEAD").exists());

        dir
    }

    #[test]
    fn test_merge_state_no_merge() {
        let (dir, _repo) = create_test_repo();

        let state = get_merge_state(dir.path()).expect("merge state should load");
        assert!(!state.is_merging);
        assert!(state.incoming_sha.is_none());
        assert!(state.conflicted_files.is_empty());
    }

    #[test]
    fn test_merge_state_with_conflict() {
        let dir = create_merge_conflict_repo();
        let state = get_merge_state(dir.path()).expect("merge state should load");

        assert!(state.is_merging);
        assert!(state.incoming_sha.is_some());
        assert!(state
            .conflicted_files
            .iter()
            .any(|file| file.path == "initial.txt"));
    }

    #[test]
    fn test_conflict_content_reads_stages() {
        let dir = create_merge_conflict_repo();

        let content = get_conflict_content(dir.path(), "initial.txt")
            .expect("conflict content should be available");

        assert_eq!(content.file_path, "initial.txt");
        assert_eq!(content.base.as_deref(), Some("initial content\n"));
        assert_eq!(content.ours, "main content\n");
        assert_eq!(content.theirs, "feature content\n");
    }

    #[test]
    fn test_conflict_content_file_not_found() {
        let dir = create_merge_conflict_repo();

        let err = get_conflict_content(dir.path(), "README.md").expect_err("should be not found");
        assert!(matches!(err, GitError::NotFound(_)));
    }

    #[test]
    fn test_merge_state_no_conflicts_returns_empty() {
        let dir = create_merge_no_conflict_no_commit_repo();

        let state = get_merge_state(dir.path()).expect("merge state should load");
        assert!(state.is_merging);
        assert!(state.incoming_sha.is_some());
        assert!(state.conflicted_files.is_empty());
    }

    #[test]
    fn test_resolve_conflict_writes_and_stages() {
        let dir = create_merge_conflict_repo();
        let path = dir.path();

        // Resolve the conflict with custom content
        resolve_conflict(path, "initial.txt", "resolved content\n")
            .expect("resolve should succeed");

        // Verify the working tree file has the resolved content
        let file_content = std::fs::read_to_string(path.join("initial.txt"))
            .expect("file should exist");
        assert_eq!(file_content, "resolved content\n");

        // Verify the file is no longer in conflict
        let state = get_merge_state(path).expect("merge state should load");
        assert!(state.is_merging); // Still merging (not committed yet)
        assert!(
            !state.conflicted_files.iter().any(|f| f.path == "initial.txt"),
            "file should no longer be conflicted after resolve"
        );
    }

    #[test]
    fn test_abort_merge_restores_head() {
        let dir = create_merge_conflict_repo();
        let path = dir.path();

        // Verify we're in a merge state
        assert!(path.join(".git/MERGE_HEAD").exists());

        // Abort the merge
        abort_merge(path).expect("abort should succeed");

        // Verify merge state is cleaned up
        assert!(!path.join(".git/MERGE_HEAD").exists());
        assert!(!path.join(".git/MERGE_MSG").exists());

        // Verify working tree is restored to HEAD
        let file_content = std::fs::read_to_string(path.join("initial.txt"))
            .expect("file should exist");
        assert_eq!(file_content, "main content\n");

        // Verify no longer merging
        let state = get_merge_state(path).expect("merge state should load");
        assert!(!state.is_merging);
    }

    #[test]
    fn test_complete_merge_creates_commit() {
        let dir = create_merge_conflict_repo();
        let path = dir.path();

        // First resolve the conflict
        resolve_conflict(path, "initial.txt", "resolved content\n")
            .expect("resolve should succeed");

        // Complete the merge
        let sha = complete_merge(path, Some("test merge commit"))
            .expect("complete merge should succeed");

        // Verify commit was created (SHA is non-empty)
        assert!(!sha.is_empty());

        // Verify merge state is cleaned up
        assert!(!path.join(".git/MERGE_HEAD").exists());

        // Verify no longer merging
        let state = get_merge_state(path).expect("merge state should load");
        assert!(!state.is_merging);

        // Verify the commit message
        let output = run_git(path, &["log", "-1", "--format=%s"]);
        let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert_eq!(msg, "test merge commit");

        // Verify it's a merge commit (2 parents)
        let output = run_git(path, &["log", "-1", "--format=%P"]);
        let parent_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parents: Vec<&str> = parent_str.split_whitespace().collect();
        assert_eq!(parents.len(), 2, "merge commit should have 2 parents");
    }

    #[test]
    fn test_complete_merge_fails_with_unresolved() {
        let dir = create_merge_conflict_repo();
        let path = dir.path();

        // Try to complete merge without resolving — should fail
        let err = complete_merge(path, None)
            .expect_err("should fail with unresolved conflicts");
        assert!(
            matches!(err, GitError::InvalidArgument(_)),
            "expected InvalidArgument, got: {err:?}"
        );
    }

    #[test]
    fn test_complete_merge_fails_without_merge() {
        let (dir, _repo) = create_test_repo();

        // Try to complete merge when no merge in progress
        let err = complete_merge(dir.path(), None)
            .expect_err("should fail without merge");
        assert!(
            matches!(err, GitError::InvalidArgument(_)),
            "expected InvalidArgument, got: {err:?}"
        );
    }

    #[test]
    fn test_complete_merge_uses_default_message() {
        let dir = create_merge_conflict_repo();
        let path = dir.path();

        // Resolve the conflict
        resolve_conflict(path, "initial.txt", "resolved\n")
            .expect("resolve should succeed");

        // Complete merge without providing a message (should use MERGE_MSG)
        let sha = complete_merge(path, None)
            .expect("complete merge should succeed");
        assert!(!sha.is_empty());

        // The default MERGE_MSG from git merge should be used
        let output = run_git(path, &["log", "-1", "--format=%s"]);
        let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
        assert!(
            msg.contains("Merge") || msg.contains("merge"),
            "default commit message should mention merge, got: {msg}"
        );
    }
}
