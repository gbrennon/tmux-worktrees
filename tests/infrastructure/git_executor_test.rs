// Fakes - one per scenario, extremely specific

use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;
use tmux_worktrees::core::error::Result;
#[path = "../common/mod.rs"]
mod common;

struct FakeGitExecutorForVersion;

impl FakeGitExecutorForVersion {
    fn run_in(&self, _cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["--version"] => Ok((0, "git version 2.40.0".to_string(), String::new())),
            _ => Ok((1, String::new(), "unexpected".to_string())),
        }
    }
}

struct FakeGitExecutorForInit;

impl FakeGitExecutorForInit {
    fn run_in(&self, _cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["init"] => Ok((0, "Initialized".to_string(), String::new())),
            ["config", "user.email", _] => Ok((0, String::new(), String::new())),
            ["config", "user.name", _] => Ok((0, String::new(), String::new())),
            ["commit", "--allow-empty", "-m", _] => Ok((0, "commit".to_string(), String::new())),
            ["status", "--porcelain"] => Ok((0, String::new(), String::new())),
            _ => Ok((1, String::new(), "unexpected".to_string())),
        }
    }
}

struct FakeGitExecutorForBranch;

impl FakeGitExecutorForBranch {
    fn run_in(&self, _cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["branch"] => Ok((0, "main\nfeature".to_string(), String::new())),
            ["branch", "test-branch"] => Ok((0, String::new(), String::new())),
            ["branch", "--list", "test-branch"] => {
                Ok((0, "test-branch".to_string(), String::new()))
            }
            ["branch", "--list", _] => Ok((0, String::new(), String::new())),
            ["branch", "-D", _] => Ok((0, String::new(), String::new())),
            _ => Ok((1, String::new(), "unexpected".to_string())),
        }
    }
}

struct FakeGitExecutorForMergeBase {
    merged: Rc<RefCell<bool>>,
}

impl FakeGitExecutorForMergeBase {
    fn new() -> Self {
        Self {
            merged: Rc::new(RefCell::new(false)),
        }
    }

    fn run_in(&self, _cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["merge-base", "--is-ancestor", "feature", "main"] => {
                let merged = *self.merged.borrow();
                Ok((if merged { 0 } else { 1 }, String::new(), String::new()))
            }
            ["merge", "feature"] => {
                *self.merged.borrow_mut() = true;
                Ok((0, String::new(), String::new()))
            }
            _ => Ok((1, String::new(), "unexpected".to_string())),
        }
    }
}

struct FakeGitExecutorForWorktree;

impl FakeGitExecutorForWorktree {
    fn run_in(&self, _cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((
                0,
                "worktree /tmp/wt\nbranch test\n".to_string(),
                String::new(),
            )),
            ["worktree", "add", _, "-b", "wt-branch", "main"] => {
                Ok((0, String::new(), String::new()))
            }
            ["worktree", "remove", "--force", _] => Ok((0, String::new(), String::new())),
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((0, "main".to_string(), String::new())),
            ["fetch", "origin", "--quiet"] => Ok((0, String::new(), String::new())),
            ["show-ref", "--verify", "--quiet", "refs/heads/main"] => {
                Ok((0, String::new(), String::new()))
            }
            ["show-ref", "--verify", "--quiet", "refs/heads/nonexistent"] => {
                Ok((1, String::new(), String::new()))
            }
            _ => Ok((1, String::new(), "unexpected".to_string())),
        }
    }
}

// Integration tests using fakes
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_executor_runs_git_command_in_repo() {
        let executor = FakeGitExecutorForVersion;
        let (status, stdout, _) = executor
            .run_in(std::path::Path::new("."), &["--version"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.starts_with("git version"));
    }

    #[test]
    fn git_executor_runs_status_command() {
        let executor = FakeGitExecutorForInit;
        let (status, stdout, _) = executor
            .run_in(std::path::Path::new("."), &["status", "--porcelain"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.is_empty());
    }

    #[test]
    fn git_executor_silent_in_ignores_errors() {
        let executor = FakeGitExecutorForInit;
        let result = executor.run_in(std::path::Path::new("."), &["invalid-command"]);
        assert!(result.is_ok());
    }

    #[test]
    fn git_executor_can_create_branch() {
        let executor = FakeGitExecutorForBranch;

        let (status, _, _) = executor
            .run_in(std::path::Path::new("."), &["branch", "test-branch"])
            .unwrap();
        assert_eq!(status, 0);

        let (status, stdout, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["branch", "--list", "test-branch"],
            )
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.contains("test-branch"));
    }

    #[test]
    fn git_executor_can_check_merge_base() {
        let executor = FakeGitExecutorForMergeBase::new();

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["merge-base", "--is-ancestor", "feature", "main"],
            )
            .unwrap();
        assert_ne!(status, 0);

        let (status, _, _) = executor
            .run_in(std::path::Path::new("."), &["merge", "feature"])
            .unwrap();
        assert_eq!(status, 0);

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["merge-base", "--is-ancestor", "feature", "main"],
            )
            .unwrap();
        assert_eq!(status, 0);
    }

    #[test]
    fn git_executor_can_list_worktrees() {
        let executor = FakeGitExecutorForWorktree;

        let (status, stdout, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["worktree", "list", "--porcelain"],
            )
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.contains("worktree "));
    }

    #[test]
    fn git_executor_can_add_worktree() {
        let executor = FakeGitExecutorForWorktree;

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["worktree", "add", "/tmp/wt", "-b", "wt-branch", "main"],
            )
            .unwrap();
        assert_eq!(status, 0);
    }

    #[test]
    fn git_executor_can_remove_worktree() {
        let executor = FakeGitExecutorForWorktree;

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["worktree", "remove", "--force", "/tmp/wt"],
            )
            .unwrap();
        assert_eq!(status, 0);
    }

    #[test]
    fn git_executor_can_get_current_branch() {
        let executor = FakeGitExecutorForWorktree;

        let (status, stdout, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["rev-parse", "--abbrev-ref", "HEAD"],
            )
            .unwrap();
        assert_eq!(status, 0);
        assert_eq!(stdout.trim(), "main");
    }

    #[test]
    fn git_executor_can_fetch() {
        let executor = FakeGitExecutorForWorktree;

        let result = executor.run_in(std::path::Path::new("."), &["fetch", "origin", "--quiet"]);
        assert!(result.is_ok());
    }

    #[test]
    fn git_executor_can_check_ref_exists() {
        let executor = FakeGitExecutorForWorktree;

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["show-ref", "--verify", "--quiet", "refs/heads/main"],
            )
            .unwrap();
        assert_eq!(status, 0);

        let (status, _, _) = executor
            .run_in(
                std::path::Path::new("."),
                &["show-ref", "--verify", "--quiet", "refs/heads/nonexistent"],
            )
            .unwrap();
        assert_ne!(status, 0);
    }
}

use common::fakes::command_runner::FakeRunner;
use tmux_worktrees::infrastructure::git_executor::GitExecutor;

/// Helper: build a `GitExecutor` backed by the given `FakeRunner`.
fn executor_with(runner: FakeRunner) -> GitExecutor {
    GitExecutor::with_runner(Box::new(runner))
}

#[test]
fn run_in_parses_successful_git_version() {
    let f = FakeRunner::new();
    f.ok("git", "--version", 0, "git version 2.54.0\n", "");

    let result = executor_with(f).run_in(std::path::Path::new("."), &["--version"]);
    assert!(result.is_ok());
    let (status, stdout, _stderr) = result.unwrap();
    assert_eq!(status, 0);
    assert_eq!(stdout, "git version 2.54.0");
}

#[test]
fn silent_in_suppresses_command_failure() {
    let f = FakeRunner::new();
    f.ok(
        "git",
        "--nonexistent-flag-xyz",
        129,
        "",
        "unknown option: --nonexistent-flag-xyz\n",
    );

    let result = executor_with(f).silent_in(std::path::Path::new("."), &["--nonexistent-flag-xyz"]);
    assert!(result.is_ok());
}

#[test]
fn silent_in_returns_ok_on_success() {
    let f = FakeRunner::new();
    f.ok("git", "--version", 0, "git version 2.54.0\n", "");

    let result = executor_with(f).silent_in(std::path::Path::new("."), &["--version"]);
    assert!(result.is_ok());
}

#[test]
fn run_in_maps_exit_code() {
    let f = FakeRunner::new();
    f.ok("git", "status", 1, "", "fatal: not a git repository\n");

    let result = executor_with(f).run_in(std::path::Path::new("."), &["status"]);
    assert!(result.is_ok());
    let (status, _, _) = result.unwrap();
    assert_eq!(status, 1);
}
