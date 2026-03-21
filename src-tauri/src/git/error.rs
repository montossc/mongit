use serde::Serialize;

/// Unified error type for all git operations in mongit.
///
/// Covers errors from git2 (read path), CLI invocations (write path),
/// repository discovery, and I/O operations.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// Error from the git2 library (read operations)
    #[error("git2: {0}")]
    Git2(#[from] git2::Error),

    /// Structured error from git CLI execution (write operations).
    /// Captures the command, stderr output, and exit code for diagnostics.
    #[error("git cli failed: `git {cmd}` (exit {exit_code:?}): {stderr}")]
    CommandFailed {
        cmd: String,
        stderr: String,
        exit_code: Option<i32>,
    },

    /// Repository or ref not found
    #[error("not found: {0}")]
    NotFound(String),

    /// Invalid argument passed to a git operation
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// Git binary not found on the system
    #[error("git not found: {message}")]
    GitNotFound { message: String },

    /// Git binary version is too old
    #[error("git version {found} is too old (minimum: {minimum})")]
    GitVersionTooOld { found: String, minimum: String },

    /// Branch operation error (structured, serializable for frontend)
    #[error("{0}")]
    BranchOp(#[from] BranchOpError),

    /// Staging operation error (structured, serializable for frontend)
    #[error("{0}")]
    StageOp(#[from] StageOpError),

    /// Filesystem I/O error
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Convert GitError to String for Tauri IPC serialization.
/// Tauri commands return `Result<T, String>`, so this bridge is required.
/// Branch operation errors are serialized as JSON for typed frontend parsing.
impl From<GitError> for String {
    fn from(err: GitError) -> Self {
        match &err {
            GitError::BranchOp(branch_err) => {
                serde_json::to_string(branch_err).unwrap_or_else(|_| err.to_string())
            }
            GitError::StageOp(stage_err) => {
                serde_json::to_string(stage_err).unwrap_or_else(|_| err.to_string())
            }
            _ => err.to_string(),
        }
    }
}

// ── Branch Operation Errors ─────────────────────────────────────────────

/// Structured error for branch operations.
///
/// Serializes as a discriminated union for typed frontend consumption:
/// `{ "kind": "BranchAlreadyExists", "branch": "...", "message": "..." }`
///
/// Raw stderr is always preserved in the `message` field for debugging.
/// Patterns derived from GitHub Desktop's dugite error mapping.
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "kind")]
pub enum BranchOpError {
    #[error("branch '{branch}' already exists")]
    BranchAlreadyExists { branch: String, message: String },

    #[error("'{name}' is not a valid branch name")]
    InvalidBranchName { name: String, message: String },

    #[error("branch '{branch}' not found")]
    BranchNotFound { branch: String, message: String },

    #[error("working tree has uncommitted changes")]
    DirtyWorkingTree { message: String },

    #[error("branch '{branch}' is not fully merged")]
    BranchNotFullyMerged { branch: String, message: String },

    #[error("cannot delete current HEAD branch '{branch}'")]
    DeleteCurrentBranch { branch: String, message: String },

    #[error("network error: {message}")]
    NetworkError { message: String },

    #[error("authentication failed: {message}")]
    AuthFailure { message: String },

    #[error("remote '{remote}' not found")]
    RemoteNotFound { remote: String, message: String },

    #[error("merge conflicts in {count} file(s)")]
    MergeConflicts { count: usize, message: String },

    #[error("branches have diverged")]
    BranchesDiverged { message: String },

    #[error("no upstream branch configured")]
    NoUpstreamBranch { message: String },

    #[error("push rejected: non-fast-forward")]
    PushNonFastForward { message: String },

    #[error("remote rejected push to protected branch")]
    ProtectedBranch { message: String },

    #[error("git command failed: {stderr}")]
    GenericCommandFailed {
        cmd: String,
        stderr: String,
        exit_code: Option<i32>,
    },
}

// ── Staging Operation Errors ────────────────────────────────────────────

/// Structured error for hunk staging operations.
///
/// Serializes as a discriminated union for typed frontend consumption:
/// `{ "kind": "PatchFailed", "reason": "...", "message": "..." }`
///
/// Raw stderr is always preserved in the `message` field for debugging.
#[derive(Debug, Clone, Serialize, thiserror::Error)]
#[serde(tag = "kind")]
pub enum StageOpError {
    #[error("patch cannot be applied: {reason}")]
    PatchFailed { reason: String, message: String },

    #[error("hunk index {index} out of range (file has {total} hunks)")]
    InvalidHunkIndex {
        index: usize,
        total: usize,
        message: String,
    },

    #[error("file '{path}' not found in diff")]
    FileNotInDiff { path: String, message: String },

    #[error("binary file '{path}' not supported for partial staging")]
    BinaryNotSupported { path: String, message: String },

    #[error("staging operation failed: {stderr}")]
    GenericStageFailed {
        cmd: String,
        stderr: String,
        exit_code: Option<i32>,
    },

    #[error("invalid line selection: {reason}")]
    InvalidLineSelection { reason: String, message: String },
}

/// Parse git CLI stderr into a typed `StageOpError`.
///
/// Matches the most common `git apply` error patterns.
/// Falls back to `GenericStageFailed` for unrecognized stderr output.
pub fn parse_stage_stderr(cmd: &str, stderr: &str, exit_code: Option<i32>) -> StageOpError {
    let lower = stderr.to_lowercase();

    if lower.contains("patch does not apply") || lower.contains("does not apply") {
        return StageOpError::PatchFailed {
            reason: "patch does not apply to current state".to_string(),
            message: stderr.to_string(),
        };
    }

    if lower.contains("corrupt patch") || lower.contains("invalid patch") {
        return StageOpError::PatchFailed {
            reason: "corrupt or invalid patch format".to_string(),
            message: stderr.to_string(),
        };
    }

    if lower.contains("binary") && lower.contains("patch") {
        let path = extract_quoted(stderr).unwrap_or_default();
        return StageOpError::BinaryNotSupported {
            path,
            message: stderr.to_string(),
        };
    }

    StageOpError::GenericStageFailed {
        cmd: cmd.to_string(),
        stderr: stderr.to_string(),
        exit_code,
    }
}

/// Parse git CLI stderr into a typed `BranchOpError`.
///
/// Matches the ~15 most common git error patterns (derived from GitHub
/// Desktop's dugite error mapping). Falls back to `GenericCommandFailed`
/// for unrecognized stderr output.
pub fn parse_branch_stderr(cmd: &str, stderr: &str, exit_code: Option<i32>) -> BranchOpError {
    let lower = stderr.to_lowercase();

    // Branch already exists
    if lower.contains("already exists") {
        return BranchOpError::BranchAlreadyExists {
            branch: extract_quoted(stderr).unwrap_or_default(),
            message: stderr.to_string(),
        };
    }

    // Invalid branch name
    if lower.contains("not a valid branch name") || lower.contains("is not a valid ref name") {
        return BranchOpError::InvalidBranchName {
            name: extract_quoted(stderr).unwrap_or_default(),
            message: stderr.to_string(),
        };
    }

    // Cannot delete checked-out branch (check before generic "not found")
    if lower.contains("cannot delete branch") && lower.contains("checked out") {
        return BranchOpError::DeleteCurrentBranch {
            branch: extract_quoted(stderr).unwrap_or_default(),
            message: stderr.to_string(),
        };
    }

    // Not fully merged
    if lower.contains("not fully merged") {
        return BranchOpError::BranchNotFullyMerged {
            branch: extract_quoted(stderr).unwrap_or_default(),
            message: stderr.to_string(),
        };
    }

    // Branch not found
    if lower.contains("branch") && lower.contains("not found") {
        return BranchOpError::BranchNotFound {
            branch: extract_quoted(stderr).unwrap_or_default(),
            message: stderr.to_string(),
        };
    }

    // Dirty working tree
    if (lower.contains("local changes") && lower.contains("overwritten"))
        || lower.contains("uncommitted changes")
    {
        return BranchOpError::DirtyWorkingTree {
            message: stderr.to_string(),
        };
    }

    // Auth failure (check before network errors — both can contain "unable to access")
    if lower.contains("authentication failed")
        || lower.contains("permission denied")
        || (lower.contains("could not read") && lower.contains("terminal"))
    {
        return BranchOpError::AuthFailure {
            message: stderr.to_string(),
        };
    }

    // Network error
    if lower.contains("could not resolve host")
        || lower.contains("unable to access")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
    {
        return BranchOpError::NetworkError {
            message: stderr.to_string(),
        };
    }

    // Remote not found
    if lower.contains("does not appear to be a git repository") || lower.contains("no such remote")
    {
        return BranchOpError::RemoteNotFound {
            remote: extract_quoted(stderr).unwrap_or_else(|| "origin".to_string()),
            message: stderr.to_string(),
        };
    }

    // Merge conflicts
    if lower.contains("conflict")
        && (lower.contains("merge") || lower.contains("automatic merge failed"))
    {
        return BranchOpError::MergeConflicts {
            count: count_conflicts(stderr),
            message: stderr.to_string(),
        };
    }

    // Divergent branches
    if lower.contains("divergent branches") || lower.contains("have diverged") {
        return BranchOpError::BranchesDiverged {
            message: stderr.to_string(),
        };
    }

    // No upstream branch
    if lower.contains("no tracking information")
        || lower.contains("no upstream branch")
        || lower.contains("does not track")
    {
        return BranchOpError::NoUpstreamBranch {
            message: stderr.to_string(),
        };
    }

    // Push non-fast-forward
    if lower.contains("non-fast-forward")
        || (lower.contains("rejected") && lower.contains("fetch first"))
    {
        return BranchOpError::PushNonFastForward {
            message: stderr.to_string(),
        };
    }

    // Protected branch
    if lower.contains("protected branch") {
        return BranchOpError::ProtectedBranch {
            message: stderr.to_string(),
        };
    }

    // Fallback — preserve raw stderr for debugging
    BranchOpError::GenericCommandFailed {
        cmd: cmd.to_string(),
        stderr: stderr.to_string(),
        exit_code,
    }
}

/// Extract the first single-quoted string from a message.
/// Git formats errors like: `fatal: A branch named 'foo' already exists.`
fn extract_quoted(s: &str) -> Option<String> {
    let start = s.find('\'')?;
    let rest = &s[start + 1..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

/// Count CONFLICT lines in merge output. Returns at least 1 if called.
fn count_conflicts(stderr: &str) -> usize {
    stderr
        .lines()
        .filter(|l| l.contains("CONFLICT"))
        .count()
        .max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_parsing_branch_already_exists() {
        let err = parse_branch_stderr(
            "branch feature-x",
            "fatal: A branch named 'feature-x' already exists.",
            Some(128),
        );
        assert!(
            matches!(err, BranchOpError::BranchAlreadyExists { ref branch, .. } if branch == "feature-x")
        );
    }

    #[test]
    fn test_error_parsing_invalid_branch_name() {
        let err = parse_branch_stderr(
            "branch ..bad",
            "fatal: '..bad' is not a valid branch name",
            Some(128),
        );
        assert!(
            matches!(err, BranchOpError::InvalidBranchName { ref name, .. } if name == "..bad")
        );
    }

    #[test]
    fn test_error_parsing_branch_not_found() {
        let err = parse_branch_stderr("branch -d gone", "error: branch 'gone' not found.", Some(1));
        assert!(
            matches!(err, BranchOpError::BranchNotFound { ref branch, .. } if branch == "gone")
        );
    }

    #[test]
    fn test_error_parsing_delete_current_branch() {
        let err = parse_branch_stderr(
            "branch -d main",
            "error: Cannot delete branch 'main' checked out at '/repo'",
            Some(1),
        );
        assert!(
            matches!(err, BranchOpError::DeleteCurrentBranch { ref branch, .. } if branch == "main")
        );
    }

    #[test]
    fn test_error_parsing_not_fully_merged() {
        let err = parse_branch_stderr(
            "branch -d feature",
            "error: The branch 'feature' is not fully merged.",
            Some(1),
        );
        assert!(
            matches!(err, BranchOpError::BranchNotFullyMerged { ref branch, .. } if branch == "feature")
        );
    }

    #[test]
    fn test_error_parsing_dirty_working_tree() {
        let err = parse_branch_stderr(
            "switch main",
            "error: Your local changes to the following files would be overwritten by checkout:\n\tfile.txt",
            Some(1),
        );
        assert!(matches!(err, BranchOpError::DirtyWorkingTree { .. }));
    }

    #[test]
    fn test_error_parsing_auth_failure() {
        let err = parse_branch_stderr(
            "push origin main",
            "fatal: Authentication failed for 'https://github.com/user/repo.git/'",
            Some(128),
        );
        assert!(matches!(err, BranchOpError::AuthFailure { .. }));
    }

    #[test]
    fn test_error_parsing_network_error() {
        let err = parse_branch_stderr(
            "fetch origin",
            "fatal: unable to access 'https://github.com/user/repo.git/': Could not resolve host: github.com",
            Some(128),
        );
        assert!(matches!(err, BranchOpError::NetworkError { .. }));
    }

    #[test]
    fn test_error_parsing_remote_not_found() {
        let err = parse_branch_stderr(
            "fetch origin",
            "fatal: 'origin' does not appear to be a git repository",
            Some(128),
        );
        assert!(
            matches!(err, BranchOpError::RemoteNotFound { ref remote, .. } if remote == "origin")
        );
    }

    #[test]
    fn test_error_parsing_merge_conflicts() {
        let err = parse_branch_stderr(
            "pull origin main",
            "CONFLICT (content): Merge conflict in file.txt\nAutomatic merge failed; fix conflicts and then commit the result.",
            Some(1),
        );
        assert!(matches!(
            err,
            BranchOpError::MergeConflicts { count: 1, .. }
        ));
    }

    #[test]
    fn test_error_parsing_divergent_branches() {
        let err = parse_branch_stderr(
            "pull origin main",
            "fatal: Need to specify how to reconcile divergent branches.",
            Some(128),
        );
        assert!(matches!(err, BranchOpError::BranchesDiverged { .. }));
    }

    #[test]
    fn test_error_parsing_no_upstream() {
        let err = parse_branch_stderr(
            "pull",
            "There is no tracking information for the current branch.",
            Some(1),
        );
        assert!(matches!(err, BranchOpError::NoUpstreamBranch { .. }));
    }

    #[test]
    fn test_error_parsing_push_non_fast_forward() {
        let err = parse_branch_stderr(
            "push origin main",
            " ! [rejected]        main -> main (non-fast-forward)\nerror: failed to push some refs to 'origin'",
            Some(1),
        );
        assert!(matches!(err, BranchOpError::PushNonFastForward { .. }));
    }

    #[test]
    fn test_error_parsing_protected_branch() {
        let err = parse_branch_stderr(
            "push origin main",
            "remote: error: GH006: Protected branch update failed.\nremote: Cannot force-push to a protected branch",
            Some(1),
        );
        assert!(matches!(err, BranchOpError::ProtectedBranch { .. }));
    }

    #[test]
    fn test_error_parsing_generic_fallback() {
        let err = parse_branch_stderr("weird-command", "some unknown error happened", Some(1));
        assert!(matches!(err, BranchOpError::GenericCommandFailed { .. }));
    }

    #[test]
    fn test_error_parsing_serializes_as_json() {
        let err = BranchOpError::BranchAlreadyExists {
            branch: "feature-x".to_string(),
            message: "branch already exists".to_string(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"BranchAlreadyExists\""));
        assert!(json.contains("\"branch\":\"feature-x\""));
    }

    #[test]
    fn test_error_git_error_branch_op_serializes() {
        let branch_err = BranchOpError::BranchNotFound {
            branch: "gone".to_string(),
            message: "not found".to_string(),
        };
        let git_err = GitError::BranchOp(branch_err);
        let s: String = git_err.into();
        // Should be JSON, not Display string
        assert!(s.contains("BranchNotFound"));
        assert!(s.contains("gone"));
    }

    #[test]
    fn test_extract_quoted_basic() {
        assert_eq!(
            extract_quoted("branch 'foo' exists"),
            Some("foo".to_string())
        );
    }

    #[test]
    fn test_extract_quoted_none() {
        assert_eq!(extract_quoted("no quotes here"), None);
    }

    #[test]
    fn test_extract_quoted_first_match() {
        assert_eq!(extract_quoted("'a' and 'b'"), Some("a".to_string()));
    }
}
