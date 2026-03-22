use std::collections::HashSet;
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
}
