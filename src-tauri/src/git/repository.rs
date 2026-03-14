use git2::{BranchType, Delta, DiffOptions, ErrorCode, Sort, Status, StatusOptions};
use serde::Serialize;

use super::GitError;

#[derive(Debug, Clone, Serialize)]
pub struct RepoStatusInfo {
    pub changed_files: usize,
    pub staged_files: usize,
}

#[derive(Debug, Clone, Serialize)]
pub enum DiffFileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffLineInfo {
    pub origin: char,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffHunkInfo {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub header: String,
    pub lines: Vec<DiffLineInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffFileEntry {
    pub path: String,
    pub status: DiffFileStatus,
    pub hunks: Vec<DiffHunkInfo>,
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

pub struct Git2Repository;

impl Git2Repository {
    pub fn status(path: &str) -> Result<RepoStatusInfo, GitError> {
        let repo = git2::Repository::open(path)?;

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

    pub fn diff_workdir(path: &str) -> Result<Vec<DiffFileEntry>, GitError> {
        use std::cell::RefCell;

        let repo = git2::Repository::open(path)?;

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

    pub fn log(path: &str, max_count: usize) -> Result<Vec<CommitInfo>, GitError> {
        let repo = git2::Repository::open(path)?;
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

    pub fn log_all_branches(path: &str, max_count: usize) -> Result<Vec<CommitInfo>, GitError> {
        let repo = git2::Repository::open(path)?;
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

    pub fn branches(path: &str) -> Result<Vec<BranchInfo>, GitError> {
        let repo = git2::Repository::open(path)?;
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

    pub fn refs(path: &str) -> Result<Vec<RefInfo>, GitError> {
        let repo = git2::Repository::open(path)?;
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

    pub fn current_branch(path: &str) -> Result<Option<String>, GitError> {
        let repo = git2::Repository::open(path)?;
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
}
