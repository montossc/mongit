use std::path::{Path, PathBuf};

use git2::{BranchType, Delta, DiffOptions, ErrorCode, Sort, Status, StatusOptions};
use serde::{Deserialize, Serialize};

use super::GitError;

// ── Data types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RepoStatusInfo {
    pub changed_files: usize,
    pub staged_files: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum DiffFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

/// Classification of how a single file changed.
/// Used for the dual-state changed-files model (staged + unstaged per row).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Typechange,
    Conflicted,
}

/// A changed file with separate staged and unstaged status.
/// This is the canonical row model for the changes workspace.
/// Both `staged` and `unstaged` are Option — at least one will be Some.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ChangedFileEntry {
    /// Relative path within the repo (forward-slash separated)
    pub path: String,
    /// Status in the index (staged changes), None if no staged changes
    pub staged: Option<FileChangeKind>,
    /// Status in the working tree (unstaged changes), None if no unstaged changes
    pub unstaged: Option<FileChangeKind>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct DiffLineInfo {
    pub origin: char,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct DiffHunkInfo {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLineInfo>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct DiffFileEntry {
    pub path: String,
    pub status: DiffFileStatus,
    pub hunks: Vec<DiffHunkInfo>,
}

/// Pair of file contents for diff rendering: original (from HEAD) and modified (from working tree).
#[derive(Debug, Clone, Serialize)]
pub struct FileContentPair {
    pub original: String,
    pub modified: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author_name: String,
    pub author_email: String,
    pub time: i64,
    pub parent_ids: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub struct BranchInfo {
    pub name: String,
    pub is_head: bool,
    pub target: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RefInfo {
    pub name: String,
    pub ref_type: RefType,
    pub commit_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub enum RefType {
    LocalBranch,
    RemoteBranch,
    Tag,
    Head,
}

// ── GitRepository trait ─────────────────────────────────────────────────

/// Trait abstracting read-only git operations.
///
/// All methods take `&self` — implementations store the repo path internally
/// and open the repository per-call (no shared `Repository` state).
///
/// This enables:
/// - Mock implementations for testing
/// - Trait objects for dependency injection in commands
/// - Clean separation between read (git2) and write (CLI) paths
#[allow(dead_code)]
pub trait GitRepository: Send + Sync {
    /// Working directory status (changed + staged file counts).
    fn status(&self) -> Result<RepoStatusInfo, GitError>;

    fn changed_files(&self) -> Result<Vec<ChangedFileEntry>, GitError>;

    /// Diff of working directory against the index.
    fn diff_workdir(&self) -> Result<Vec<DiffFileEntry>, GitError>;

    /// Diff of the index against HEAD (staged changes).
    fn diff_index(&self) -> Result<Vec<DiffFileEntry>, GitError>;

    /// Commit log from HEAD, topological + time ordered.
    fn log(&self, max_count: usize) -> Result<Vec<CommitInfo>, GitError>;

    /// Commit log from all branches (local + remote).
    fn log_all_branches(&self, max_count: usize) -> Result<Vec<CommitInfo>, GitError>;

    /// List local branches.
    fn branches(&self) -> Result<Vec<BranchInfo>, GitError>;

    /// List all refs (HEAD, local branches, remote branches, tags).
    fn refs(&self) -> Result<Vec<RefInfo>, GitError>;

    /// Current branch name (None if detached HEAD or unborn).
    fn current_branch(&self) -> Result<Option<String>, GitError>;

    /// Get original (HEAD) and modified (working tree) content for a file.
    fn file_content_for_diff(&self, file_path: &str) -> Result<FileContentPair, GitError>;
}

// ── Git2Repository (git2 read implementation) ───────────────────────────

/// Read-only git operations backed by libgit2.
///
/// Opens the repository from `path` on every call (open-per-call pattern).
/// This avoids `Arc<Mutex<Repository>>` and the Send/Sync constraints of git2.
pub struct Git2Repository {
    path: PathBuf,
}

impl Git2Repository {
    /// Create a new Git2Repository for the given working directory.
    pub fn open(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// Access the stored path.
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Internal helper: open the git2 repository.
    fn repo(&self) -> Result<git2::Repository, GitError> {
        Ok(git2::Repository::open(&self.path)?)
    }
}

impl GitRepository for Git2Repository {
    fn status(&self) -> Result<RepoStatusInfo, GitError> {
        let repo = self.repo()?;

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_ignored(false)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        let mut changed_files = 0usize;
        let mut staged_files = 0usize;

        for entry in statuses.iter() {
            let s = entry.status();

            if s.intersects(
                Status::WT_MODIFIED
                    | Status::WT_NEW
                    | Status::WT_DELETED
                    | Status::WT_RENAMED
                    | Status::WT_TYPECHANGE,
            ) {
                changed_files += 1;
            }

            if s.intersects(
                Status::INDEX_NEW
                    | Status::INDEX_MODIFIED
                    | Status::INDEX_DELETED
                    | Status::INDEX_RENAMED
                    | Status::INDEX_TYPECHANGE,
            ) {
                staged_files += 1;
            }
        }

        Ok(RepoStatusInfo {
            changed_files,
            staged_files,
        })
    }

    fn changed_files(&self) -> Result<Vec<ChangedFileEntry>, GitError> {
        let repo = self.repo()?;
        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .renames_head_to_index(true)
            .renames_index_to_workdir(true);

        let statuses = repo.statuses(Some(&mut opts))?;

        let mut entries = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let bits = entry.status();

            // Check for conflicts first — conflicted files get special treatment
            if bits.intersects(git2::Status::CONFLICTED) {
                entries.push(ChangedFileEntry {
                    path,
                    staged: Some(FileChangeKind::Conflicted),
                    unstaged: Some(FileChangeKind::Conflicted),
                });
                continue;
            }

            let staged = if bits.intersects(git2::Status::INDEX_RENAMED) {
                Some(FileChangeKind::Renamed)
            } else if bits.intersects(git2::Status::INDEX_TYPECHANGE) {
                Some(FileChangeKind::Typechange)
            } else if bits.intersects(git2::Status::INDEX_NEW) {
                Some(FileChangeKind::Added)
            } else if bits.intersects(git2::Status::INDEX_MODIFIED) {
                Some(FileChangeKind::Modified)
            } else if bits.intersects(git2::Status::INDEX_DELETED) {
                Some(FileChangeKind::Deleted)
            } else {
                None
            };

            let unstaged = if bits.intersects(git2::Status::WT_RENAMED) {
                Some(FileChangeKind::Renamed)
            } else if bits.intersects(git2::Status::WT_TYPECHANGE) {
                Some(FileChangeKind::Typechange)
            } else if bits.intersects(git2::Status::WT_NEW) {
                Some(FileChangeKind::Added)
            } else if bits.intersects(git2::Status::WT_MODIFIED) {
                Some(FileChangeKind::Modified)
            } else if bits.intersects(git2::Status::WT_DELETED) {
                Some(FileChangeKind::Deleted)
            } else {
                None
            };

            if staged.is_some() || unstaged.is_some() {
                entries.push(ChangedFileEntry {
                    path,
                    staged,
                    unstaged,
                });
            }
        }

        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    fn diff_workdir(&self) -> Result<Vec<DiffFileEntry>, GitError> {
        use std::cell::RefCell;

        let repo = self.repo()?;

        let mut opts = DiffOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(true)
            .include_typechange(true);

        let diff = repo.diff_index_to_workdir(None, Some(&mut opts))?;
        let entries: RefCell<Vec<DiffFileEntry>> = RefCell::new(Vec::new());

        diff.foreach(
            &mut |delta, _| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let status = match delta.status() {
                    Delta::Added => DiffFileStatus::Added,
                    Delta::Deleted => DiffFileStatus::Deleted,
                    Delta::Renamed => DiffFileStatus::Renamed,
                    _ => DiffFileStatus::Modified,
                };

                entries.borrow_mut().push(DiffFileEntry {
                    path,
                    status,
                    hunks: Vec::new(),
                });

                true
            },
            None,
            Some(&mut |_delta, hunk| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    file.hunks.push(DiffHunkInfo {
                        old_start: hunk.old_start(),
                        old_lines: hunk.old_lines(),
                        new_start: hunk.new_start(),
                        new_lines: hunk.new_lines(),
                        header: String::from_utf8_lossy(hunk.header())
                            .trim_end_matches('\0')
                            .to_string(),
                        lines: Vec::new(),
                    });
                }
                true
            }),
            Some(&mut |_delta, _hunk, line| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    if let Some(hunk) = file.hunks.last_mut() {
                        hunk.lines.push(DiffLineInfo {
                            origin: line.origin(),
                            content: String::from_utf8_lossy(line.content()).to_string(),
                            old_lineno: line.old_lineno(),
                            new_lineno: line.new_lineno(),
                        });
                    }
                }
                true
            }),
        )?;

        Ok(entries.into_inner())
    }

    fn diff_index(&self) -> Result<Vec<DiffFileEntry>, GitError> {
        use std::cell::RefCell;

        let repo = self.repo()?;

        // Get HEAD tree (None for initial/unborn commits)
        let head_tree = match repo.head() {
            Ok(head) => Some(head.peel_to_tree()?),
            Err(e) if e.code() == ErrorCode::UnbornBranch => None,
            Err(e) => return Err(e.into()),
        };

        let mut opts = DiffOptions::new();
        opts.include_typechange(true);

        let diff = repo.diff_tree_to_index(
            head_tree.as_ref(),
            None, // current index
            Some(&mut opts),
        )?;

        let entries: RefCell<Vec<DiffFileEntry>> = RefCell::new(Vec::new());

        diff.foreach(
            &mut |delta, _| {
                let path = delta
                    .new_file()
                    .path()
                    .or_else(|| delta.old_file().path())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default();

                let status = match delta.status() {
                    Delta::Added => DiffFileStatus::Added,
                    Delta::Deleted => DiffFileStatus::Deleted,
                    Delta::Renamed => DiffFileStatus::Renamed,
                    _ => DiffFileStatus::Modified,
                };

                entries.borrow_mut().push(DiffFileEntry {
                    path,
                    status,
                    hunks: Vec::new(),
                });

                true
            },
            None,
            Some(&mut |_delta, hunk| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    file.hunks.push(DiffHunkInfo {
                        old_start: hunk.old_start(),
                        old_lines: hunk.old_lines(),
                        new_start: hunk.new_start(),
                        new_lines: hunk.new_lines(),
                        header: String::from_utf8_lossy(hunk.header())
                            .trim_end_matches('\0')
                            .to_string(),
                        lines: Vec::new(),
                    });
                }
                true
            }),
            Some(&mut |_delta, _hunk, line| {
                let mut entries = entries.borrow_mut();
                if let Some(file) = entries.last_mut() {
                    if let Some(hunk) = file.hunks.last_mut() {
                        hunk.lines.push(DiffLineInfo {
                            origin: line.origin(),
                            content: String::from_utf8_lossy(line.content()).to_string(),
                            old_lineno: line.old_lineno(),
                            new_lineno: line.new_lineno(),
                        });
                    }
                }
                true
            }),
        )?;

        Ok(entries.into_inner())
    }

    fn log(&self, max_count: usize) -> Result<Vec<CommitInfo>, GitError> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;
        revwalk.push_head()?;

        let mut commits = Vec::new();

        for oid_result in revwalk.take(max_count) {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();

            commits.push(CommitInfo {
                id: commit.id().to_string(),
                message: commit.message().unwrap_or_default().to_string(),
                author_name: author.name().unwrap_or_default().to_string(),
                author_email: author.email().unwrap_or_default().to_string(),
                time: commit.time().seconds(),
                parent_ids: commit
                    .parent_ids()
                    .map(|parent| parent.to_string())
                    .collect(),
            });
        }

        Ok(commits)
    }

    fn log_all_branches(&self, max_count: usize) -> Result<Vec<CommitInfo>, GitError> {
        let repo = self.repo()?;
        let mut revwalk = repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::TIME)?;

        // Push all local branch heads.
        revwalk.push_glob("refs/heads/*")?;

        // Also include remote branch heads when present.
        let _ = revwalk.push_glob("refs/remotes/*");

        let mut commits = Vec::new();

        for oid_result in revwalk.take(max_count) {
            let oid = oid_result?;
            let commit = repo.find_commit(oid)?;
            let author = commit.author();

            commits.push(CommitInfo {
                id: commit.id().to_string(),
                message: commit.message().unwrap_or_default().to_string(),
                author_name: author.name().unwrap_or_default().to_string(),
                author_email: author.email().unwrap_or_default().to_string(),
                time: commit.time().seconds(),
                parent_ids: commit
                    .parent_ids()
                    .map(|parent| parent.to_string())
                    .collect(),
            });
        }

        Ok(commits)
    }

    fn branches(&self) -> Result<Vec<BranchInfo>, GitError> {
        let repo = self.repo()?;
        let mut out = Vec::new();

        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or_default().to_string();
            let is_head = branch.is_head();
            let target = branch
                .get()
                .target()
                .map(|oid| oid.to_string())
                .unwrap_or_default();

            out.push(BranchInfo {
                name,
                is_head,
                target,
            });
        }

        Ok(out)
    }

    fn refs(&self) -> Result<Vec<RefInfo>, GitError> {
        let repo = self.repo()?;
        let mut out = Vec::new();

        if let Ok(head) = repo.head() {
            if let Some(target) = head.target() {
                out.push(RefInfo {
                    name: "HEAD".to_string(),
                    ref_type: RefType::Head,
                    commit_id: target.to_string(),
                });
            }
        }

        for branch_result in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or_default().to_string();

            if let Some(target) = branch.get().target() {
                out.push(RefInfo {
                    name,
                    ref_type: RefType::LocalBranch,
                    commit_id: target.to_string(),
                });
            }
        }

        for branch_result in repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch_result?;
            let name = branch.name()?.unwrap_or_default().to_string();

            if let Some(target) = branch.get().target() {
                out.push(RefInfo {
                    name,
                    ref_type: RefType::RemoteBranch,
                    commit_id: target.to_string(),
                });
            }
        }

        for ref_result in repo.references_glob("refs/tags/*")? {
            let reference = ref_result?;
            let name = reference.shorthand().unwrap_or_default().to_string();

            if let Ok(commit) = reference.peel_to_commit() {
                out.push(RefInfo {
                    name,
                    ref_type: RefType::Tag,
                    commit_id: commit.id().to_string(),
                });
            }
        }

        Ok(out)
    }

    fn current_branch(&self) -> Result<Option<String>, GitError> {
        let repo = self.repo()?;
        let head = match repo.head() {
            Ok(head) => head,
            Err(err)
                if err.code() == ErrorCode::UnbornBranch || err.code() == ErrorCode::NotFound =>
            {
                return Ok(None);
            }
            Err(err) => return Err(err.into()),
        };

        Ok(head.shorthand().map(|name| name.to_string()))
    }

    fn file_content_for_diff(&self, file_path: &str) -> Result<FileContentPair, GitError> {
        let repo = self.repo()?;

        // Validate that file_path stays within the repo (defense-in-depth)
        let full_path = self
            .path
            .join(file_path)
            .canonicalize()
            .unwrap_or_else(|_| self.path.join(file_path));
        let repo_root = self
            .path
            .canonicalize()
            .unwrap_or_else(|_| self.path.clone());
        if !full_path.starts_with(&repo_root) {
            return Err(GitError::InvalidArgument(format!(
                "Path '{}' escapes repository root",
                file_path
            )));
        }

        // Read modified content from working directory
        let modified = if full_path.exists() {
            std::fs::read_to_string(&full_path).unwrap_or_default()
        } else {
            String::new() // Deleted file
        };

        // Read original content from HEAD tree
        let original = match repo.head() {
            Ok(head) => {
                let tree = head.peel_to_tree()?;
                match tree.get_path(Path::new(file_path)) {
                    Ok(entry) => {
                        let blob = repo.find_blob(entry.id())?;
                        String::from_utf8_lossy(blob.content()).to_string()
                    }
                    Err(_) => String::new(), // New/untracked file
                }
            }
            Err(_) => String::new(), // No HEAD (empty repo)
        };

        Ok(FileContentPair { original, modified })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    /// Helper to create a temporary git repo with a committed file and a working-tree change.
    fn setup_repo_with_change() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path();

        // init + initial commit
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .expect("git init");
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .expect("git config email");
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(path)
            .output()
            .expect("git config name");

        std::fs::write(path.join("hello.txt"), "hello world\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .expect("git add");
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(path)
            .output()
            .expect("git commit");

        // now modify the file
        std::fs::write(path.join("hello.txt"), "hello world\nmodified line\n").unwrap();

        dir
    }

    #[test]
    fn test_diff_workdir_detects_modified_file() {
        let dir = setup_repo_with_change();
        let repo = Git2Repository::open(dir.path());
        let entries = repo.diff_workdir().expect("diff_workdir should succeed");

        assert_eq!(entries.len(), 1, "should detect exactly one changed file");
        assert_eq!(entries[0].path, "hello.txt");
        assert!(matches!(entries[0].status, DiffFileStatus::Modified));
        assert!(
            !entries[0].hunks.is_empty(),
            "should have at least one hunk"
        );

        let lines = &entries[0].hunks[0].lines;
        let added_lines: Vec<_> = lines.iter().filter(|l| l.origin == '+').collect();
        assert!(!added_lines.is_empty(), "should have added lines");
    }

    #[test]
    fn test_diff_workdir_detects_new_untracked_file() {
        let dir = setup_repo_with_change();
        std::fs::write(dir.path().join("new-file.txt"), "brand new\n").unwrap();

        let repo = Git2Repository::open(dir.path());
        let entries = repo.diff_workdir().expect("diff_workdir should succeed");

        let new_file = entries.iter().find(|e| e.path == "new-file.txt");
        assert!(new_file.is_some(), "should detect untracked file");
    }

    #[test]
    fn test_diff_workdir_empty_when_clean() {
        let dir = tempfile::tempdir().expect("create tempdir");
        let path = dir.path();

        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(path)
            .output()
            .unwrap();
        std::fs::write(path.join("file.txt"), "content\n").unwrap();
        Command::new("git")
            .args(["add", "."])
            .current_dir(path)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(path)
            .output()
            .unwrap();

        let repo = Git2Repository::open(path);
        let entries = repo.diff_workdir().expect("diff_workdir should succeed");
        assert!(entries.is_empty(), "clean repo should have no diff entries");
    }

    #[test]
    fn test_diff_workdir_invalid_path_returns_error() {
        let repo = Git2Repository::open("/nonexistent/repo/path");
        let result = repo.diff_workdir();
        assert!(result.is_err(), "invalid path should return error");
    }
}
