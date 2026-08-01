use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub struct GitExecutor;

impl GitExecutor {
    pub fn run_in(&self, cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        let out = Command::new("git")
            .current_dir(cwd)
            .args(args)
            .output()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn git_executor_runs_git_command() {
        let executor = GitExecutor;
        let result = executor.run_in(&PathBuf::from("."), &["--version"]);
        assert!(result.is_ok());
        let (status, stdout, _) = result.unwrap();
        assert_eq!(status, 0);
        assert!(stdout.starts_with("git version"));
    }
}