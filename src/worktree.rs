use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::git;

pub fn list_worktrees(repo_root: &Path, work_dir: &str) -> Result<Vec<PathBuf>> {
    let (status, out, _) = git::run_in(repo_root, &["worktree", "list", "--porcelain"])?;
    if status != 0 {
        return Ok(Vec::new());
    }
    let prefix = std::fs::canonicalize(repo_root.join(work_dir))
        .unwrap_or_else(|_| repo_root.join(work_dir));
    let mut res = Vec::new();
    for line in out.lines() {
        if let Some(p) = line.strip_prefix("worktree ") {
            let p = PathBuf::from(p.trim());
            let matches = p.starts_with(&prefix)
                || std::fs::canonicalize(&p)
                    .map(|c| c.starts_with(&prefix))
                    .unwrap_or(false);
            if matches {
                res.push(p);
            }
        }
    }
    Ok(res)
}

pub fn list_worktree_names(repo_root: &Path, work_dir: &str) -> Result<Vec<String>> {
    let mut names: Vec<String> = list_worktrees(repo_root, work_dir)?
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    Ok(names)
}

pub fn create(repo_root: &Path, target: &Path, branch: &str, base: &str) -> Result<()> {
    let _ = git::silent_in(repo_root, &["fetch", "origin", "--quiet"]);
    let mut base = base.to_string();
    let remote_base = format!("refs/remotes/origin/{base}");
    let exists = git::run_in(repo_root, &["show-ref", "--verify", "--quiet", &remote_base])
        .map(|(s, _, _)| s == 0)
        .unwrap_or(false);
    if exists {
        base = format!("origin/{base}");
    }
    let (status, stdout, stderr) = git::run_in(
        repo_root,
        &["worktree", "add", target.to_str().unwrap(), "-b", branch, &base],
    )?;
    if status != 0 {
        anyhow::bail!("{}", if stdout.is_empty() { stderr } else { stdout });
    }
    Ok(())
}

pub fn remove(repo_root: &Path, wt_dir: &Path, branch: &str) -> Result<()> {
    if !branch.is_empty() {
        crate::tmux::kill_window(branch)?;
    }
    let (status, stdout, stderr) = git::run_in(
        repo_root,
        &["worktree", "remove", "--force", wt_dir.to_str().unwrap()],
    )?;
    if status != 0 {
        anyhow::bail!("{}", if stdout.is_empty() { stderr } else { stdout });
    }
    Ok(())
}

pub fn branch(wt_dir: &Path) -> Result<String> {
    let (status, out, _) = git::run_in(wt_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if status != 0 {
        return Ok("?".to_string());
    }
    Ok(out)
}

pub fn abspath(target: &Path) -> String {
    std::fs::canonicalize(target)
        .unwrap_or_else(|_| target.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

pub fn delete_branch(repo_root: &Path, branch: &str) {
    let _ = git::silent_in(repo_root, &["branch", "-D", branch]);
}
