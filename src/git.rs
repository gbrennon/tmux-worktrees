use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

pub fn run_in(cwd: &Path, args: &[&str]) -> Result<(i32, String, String)> {
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

pub fn silent_in(cwd: &Path, args: &[&str]) -> Result<()> {
    let _ = run_in(cwd, args);
    Ok(())
}