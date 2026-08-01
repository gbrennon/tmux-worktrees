use anyhow::{Context, Result};
use std::path::Path;

use crate::git;
use crate::tmux;

pub fn find_repo_root() -> Result<String> {
    if let Ok(root) = std::env::var("TMUX_WORKTREES_ROOT") {
        let root = root.trim().to_string();
        if !root.is_empty() && Path::new(&root).join(".git").is_dir() {
            return Ok(root);
        }
    }
    if let Ok((0, out, _)) = tmux::run(&["show-environment", "-g", "MAIN_PROJECT_PATH"]) {
        if let Some(v) = out.strip_prefix("MAIN_PROJECT_PATH=") {
            let v = v.trim();
            if !v.is_empty() {
                return Ok(v.to_string());
            }
        }
    }
    let mut dir = match tmux::run(&["display-message", "-p", "#{pane_current_path}"]) {
        Ok((0, out, _)) => out.trim().to_string(),
        _ => std::env::current_dir()
            .context("Failed to get current directory")?
            .to_string_lossy()
            .into_owned(),
    };
    if dir.is_empty() {
        dir = std::env::current_dir()?
            .to_string_lossy()
            .into_owned();
    }
    loop {
        if Path::new(&dir).join(".git").is_dir() {
            return Ok(dir);
        }
        match Path::new(&dir).parent() {
            Some(parent) if parent.as_os_str() != Path::new(&dir).as_os_str() => {
                dir = parent.to_string_lossy().into_owned();
            }
            _ => break,
        }
    }
    if Path::new("/.git").is_dir() {
        return Ok("/".to_string());
    }
    anyhow::bail!("Not in a git repository")
}

pub fn resolve_default_branch(repo_root: &Path) -> Result<String> {
    for args in [
        &["config", "--global", "init.defaultBranch"][..],
        &["config", "init.defaultBranch"][..],
        &["branch", "--show-current"][..],
    ] {
        if let Ok((0, out, _)) = git::run_in(repo_root, args) {
            if !out.is_empty() {
                return Ok(out);
            }
        }
    }
    Ok("main".to_string())
}

pub fn ensure_worktrees_dir(repo_root: &Path, worktree_dir: &str) -> Result<()> {
    let dir = repo_root.join(worktree_dir);
    if !dir.is_dir() {
        std::fs::create_dir(&dir).context("Failed to create worktree directory")?;
        let exclude = repo_root.join(".git").join("info").join("exclude");
        if exclude.exists() {
            let content = std::fs::read_to_string(&exclude).unwrap_or_default();
            let line = format!("{worktree_dir}/");
            if !content.lines().any(|l| l.trim_end() == line) {
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&exclude) {
                    use std::io::Write;
                    let _ = f.write_all(format!("\n{line}\n").as_bytes());
                }
            }
        }
    }
    Ok(())
}
