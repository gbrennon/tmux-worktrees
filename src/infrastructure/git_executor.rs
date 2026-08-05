use crate::core::error::Result;
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
        let out = self.runner.run("git", args, Some(cwd)).map_err(|e| {
            crate::core::error::Error::new(format!("Failed to run git command: {}", e))
        })?;
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
