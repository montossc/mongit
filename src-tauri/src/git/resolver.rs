#![allow(dead_code)] // Public API; consumers not yet wired

use std::path::{Path, PathBuf};
use std::process::Command;

use super::GitError;

/// Minimum required git version for mongit write operations.
const MIN_GIT_VERSION: (u32, u32) = (2, 35);

/// A parsed git version (major.minor.patch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for GitVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Result of successfully resolving a git binary.
#[derive(Debug, Clone)]
pub struct ResolvedGit {
    /// Absolute path to the git executable.
    pub path: PathBuf,
    /// Parsed version of the git binary.
    pub version: GitVersion,
    /// How the git binary was discovered.
    pub source: GitSource,
}

/// How the git binary was resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitSource {
    /// Via `GIT_EXECUTABLE` environment variable.
    EnvOverride,
    /// Via system PATH lookup.
    SystemPath,
}

/// Resolves the git binary path through a deterministic priority chain.
///
/// Priority:
/// 1. `GIT_EXECUTABLE` env var (developer/CI override)
/// 2. System PATH via `which::which_global("git")`
/// 3. Error
///
/// The resolved binary is validated to be >= 2.35.
pub struct GitResolver;

impl GitResolver {
    /// Resolve the git binary, returning its path and validated version.
    pub fn resolve() -> Result<ResolvedGit, GitError> {
        // Step 1: Check GIT_EXECUTABLE env var
        if let Ok(env_path) = std::env::var("GIT_EXECUTABLE") {
            let path = PathBuf::from(&env_path);
            if path.exists() {
                // Canonicalize to absolute path to prevent relative-path hijacking
                let path = path.canonicalize().map_err(|_| GitError::GitNotFound {
                    message: format!(
                        "GIT_EXECUTABLE='{}' exists but cannot be resolved to an absolute path.",
                        env_path
                    ),
                })?;
                let version = Self::check_version(&path)?;
                Self::validate_minimum_version(&version)?;
                return Ok(ResolvedGit {
                    path,
                    version,
                    source: GitSource::EnvOverride,
                });
            }
            return Err(GitError::GitNotFound {
                message: format!(
                    "GIT_EXECUTABLE='{}' does not exist. \
                     Unset the variable or point it to a valid git binary.",
                    env_path
                ),
            });
        }

        // Step 2: System PATH lookup
        match which::which_global("git") {
            Ok(path) => {
                let version = Self::check_version(&path)?;
                Self::validate_minimum_version(&version)?;
                Ok(ResolvedGit {
                    path,
                    version,
                    source: GitSource::SystemPath,
                })
            }
            Err(_) => Err(GitError::GitNotFound {
                message: "Git not found on system PATH. \
                          Install via: brew install git"
                    .to_string(),
            }),
        }
    }

    /// Run `git --version` and parse the output.
    fn check_version(git_path: &Path) -> Result<GitVersion, GitError> {
        let output = Command::new(git_path)
            .arg("--version")
            .output()
            .map_err(|_| GitError::GitNotFound {
                message: format!(
                    "Failed to execute '{}'. Is it a valid git binary?",
                    git_path.display()
                ),
            })?;

        if !output.status.success() {
            return Err(GitError::GitNotFound {
                message: format!(
                    "'{}' returned exit code {:?}",
                    git_path.display(),
                    output.status.code()
                ),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_version(&stdout)
    }

    /// Parse a version string like "git version 2.39.3 (Apple Git-146)".
    ///
    /// Handles platform suffixes (e.g., "2.39.3.windows.1") by taking only
    /// the first three numeric components.
    fn parse_version(version_output: &str) -> Result<GitVersion, GitError> {
        let trimmed = version_output.trim();

        // Strip "git version " prefix
        let version_str = trimmed.strip_prefix("git version ").unwrap_or(trimmed);

        // Take the first word (before any space, e.g., "(Apple Git-146)")
        let first_word = version_str.split_whitespace().next().unwrap_or(version_str);

        // Split on '.' and parse numeric parts
        let parts: Vec<&str> = first_word.split('.').collect();

        let parse_part = |idx: usize| -> Result<u32, GitError> {
            parts
                .get(idx)
                .and_then(|s| s.parse::<u32>().ok())
                .ok_or_else(|| GitError::GitNotFound {
                    message: format!(
                        "Failed to parse git version from output: '{}'. \
                         Expected format: 'git version X.Y.Z'",
                        trimmed
                    ),
                })
        };

        Ok(GitVersion {
            major: parse_part(0)?,
            minor: parse_part(1)?,
            patch: parts
                .get(2)
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0),
        })
    }

    /// Validate the version meets the minimum requirement.
    fn validate_minimum_version(version: &GitVersion) -> Result<(), GitError> {
        let (min_major, min_minor) = MIN_GIT_VERSION;
        if version.major > min_major || (version.major == min_major && version.minor >= min_minor) {
            Ok(())
        } else {
            Err(GitError::GitVersionTooOld {
                found: version.to_string(),
                minimum: format!("{}.{}.0", min_major, min_minor),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mutex to serialize tests that mutate GIT_EXECUTABLE env var.
    /// Rust tests run in parallel by default — without this, env var
    /// mutations in one test can leak into another.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_parse_version_standard() {
        let v = GitResolver::parse_version("git version 2.39.3").unwrap();
        assert_eq!(
            v,
            GitVersion {
                major: 2,
                minor: 39,
                patch: 3
            }
        );
    }

    #[test]
    fn test_parse_version_apple() {
        let v = GitResolver::parse_version("git version 2.39.3 (Apple Git-146)").unwrap();
        assert_eq!(
            v,
            GitVersion {
                major: 2,
                minor: 39,
                patch: 3
            }
        );
    }

    #[test]
    fn test_parse_version_windows_suffix() {
        let v = GitResolver::parse_version("git version 2.42.0.windows.1").unwrap();
        assert_eq!(
            v,
            GitVersion {
                major: 2,
                minor: 42,
                patch: 0
            }
        );
    }

    #[test]
    fn test_parse_version_no_patch() {
        let v = GitResolver::parse_version("git version 2.35").unwrap();
        assert_eq!(
            v,
            GitVersion {
                major: 2,
                minor: 35,
                patch: 0
            }
        );
    }

    #[test]
    fn test_parse_version_with_newline() {
        let v = GitResolver::parse_version("git version 2.40.1\n").unwrap();
        assert_eq!(
            v,
            GitVersion {
                major: 2,
                minor: 40,
                patch: 1
            }
        );
    }

    #[test]
    fn test_parse_version_invalid() {
        let result = GitResolver::parse_version("not a version string");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_version_meets_minimum() {
        let v = GitVersion {
            major: 2,
            minor: 39,
            patch: 0,
        };
        assert!(GitResolver::validate_minimum_version(&v).is_ok());
    }

    #[test]
    fn test_validate_version_exact_minimum() {
        let v = GitVersion {
            major: 2,
            minor: 35,
            patch: 0,
        };
        assert!(GitResolver::validate_minimum_version(&v).is_ok());
    }

    #[test]
    fn test_validate_version_too_old() {
        let v = GitVersion {
            major: 2,
            minor: 30,
            patch: 0,
        };
        let err = GitResolver::validate_minimum_version(&v).unwrap_err();
        assert!(matches!(err, GitError::GitVersionTooOld { .. }));
    }

    #[test]
    fn test_validate_version_major_too_old() {
        let v = GitVersion {
            major: 1,
            minor: 99,
            patch: 0,
        };
        let err = GitResolver::validate_minimum_version(&v).unwrap_err();
        assert!(matches!(err, GitError::GitVersionTooOld { .. }));
    }

    #[test]
    fn test_validate_version_future_major() {
        let v = GitVersion {
            major: 3,
            minor: 0,
            patch: 0,
        };
        assert!(GitResolver::validate_minimum_version(&v).is_ok());
    }

    #[test]
    fn test_git_version_display() {
        let v = GitVersion {
            major: 2,
            minor: 39,
            patch: 3,
        };
        assert_eq!(v.to_string(), "2.39.3");
    }

    #[test]
    fn test_resolve_finds_system_git() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Ensure no leftover env var from other tests
        std::env::remove_var("GIT_EXECUTABLE");

        // This test requires git to be installed (CI and dev machines have it)
        let resolved = GitResolver::resolve().expect("git should be available");
        assert!(resolved.path.exists());
        assert!(
            resolved.version.major > 2
                || (resolved.version.major == 2 && resolved.version.minor >= 35),
            "resolved git version should be >= 2.35, got {}",
            resolved.version
        );
    }

    #[test]
    fn test_resolve_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        // Find actual git path first
        let system_git = which::which_global("git").expect("git must be on PATH");

        // Set GIT_EXECUTABLE and resolve
        std::env::set_var("GIT_EXECUTABLE", &system_git);
        let resolved = GitResolver::resolve().expect("env override should work");
        std::env::remove_var("GIT_EXECUTABLE");

        assert_eq!(resolved.source, GitSource::EnvOverride);
        // Canonicalized path may differ from raw system_git, so compare canonical forms
        assert_eq!(
            resolved.path.canonicalize().unwrap(),
            system_git.canonicalize().unwrap()
        );
    }

    #[test]
    fn test_resolve_env_override_nonexistent() {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::set_var("GIT_EXECUTABLE", "/nonexistent/git");
        let err = GitResolver::resolve().unwrap_err();
        std::env::remove_var("GIT_EXECUTABLE");

        assert!(matches!(err, GitError::GitNotFound { .. }));
    }
}
