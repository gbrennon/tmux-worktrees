use anyhow::{Context, Result};
use std::path::Path;

use crate::core::ports::GitPort;

use super::command_runner::{CommandRunner, SystemCommandRunner};

/// Thin adapter around `git` commands — every call is forwarded through the
/// injected [`CommandRunner`] so tests can supply a fake.
pub struct GitExecutor {
    runner: Box<dyn CommandRunner>,
}

impl Default for GitExecutor {
    fn default() -> Self {
        Self {
            runner: Box::new(SystemCommandRunner),
        }
    }
}

impl GitExecutor {
    #[allow(dead_code)]
    pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    pub fn run_in(&self, cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        let out = self
            .runner
            .run("git", args, Some(cwd))
            .context("Failed to run git command")?;
        Ok((
            out.status.code().unwrap_or(1),
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ))
    }

    pub fn silent_in(&self, cwd: &Path, args: &[&str]) -> Result<()> {
        let _ = self.run_in(cwd, args);
        Ok(())
    }
}

impl GitPort for GitExecutor {
    fn run_in(&self, dir: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        self.run_in(dir, args)
    }

    fn silent_in(&self, dir: &Path, args: &[&str]) -> Result<()> {
        self.silent_in(dir, args)
    }
}

// ---------------------------------------------------------------------------
// Tests — use FakeRunner with captured real output, zero shell calls.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::super::command_runner::test_support::FakeRunner;
    use super::*;
    use std::path::PathBuf;

    /// Helper: build a `GitExecutor` backed by the given `FakeRunner`.
    fn executor_with(runner: FakeRunner) -> GitExecutor {
        GitExecutor::with_runner(Box::new(runner))
    }

    #[test]
    fn run_in_parses_successful_git_version() {
        let f = FakeRunner::new();
        f.ok("git", "--version", 0, "git version 2.54.0\n", "");

        let result = executor_with(f).run_in(&PathBuf::from("."), &["--version"]);
        assert!(result.is_ok());
        let (status, stdout, _stderr) = result.unwrap();
        assert_eq!(status, 0);
        assert_eq!(stdout, "git version 2.54.0");
    }

    #[test]
    fn silent_in_suppresses_command_failure() {
        let f = FakeRunner::new();
        // Simulate a git error — silent_in must swallow it.
        f.ok(
            "git",
            "--nonexistent-flag-xyz",
            129,
            "",
            "unknown option: --nonexistent-flag-xyz\n",
        );

        let result = executor_with(f).silent_in(&PathBuf::from("."), &["--nonexistent-flag-xyz"]);
        assert!(result.is_ok());
    }

    #[test]
    fn silent_in_returns_ok_on_success() {
        let f = FakeRunner::new();
        f.ok("git", "--version", 0, "git version 2.54.0\n", "");

        let result = executor_with(f).silent_in(&PathBuf::from("."), &["--version"]);
        assert!(result.is_ok());
    }

    #[test]
    fn run_in_maps_exit_code() {
        let f = FakeRunner::new();
        f.ok("git", "status", 1, "", "fatal: not a git repository\n");

        let result = executor_with(f).run_in(&PathBuf::from("."), &["status"]);
        assert!(result.is_ok());
        let (status, _, _) = result.unwrap();
        assert_eq!(status, 1);
    }
}
