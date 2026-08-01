use anyhow::{Context, Result};
use std::process::Command;

pub fn run(args: &[&str]) -> Result<(i32, String, String)> {
    let out = Command::new("tmux")
        .args(args)
        .output()
        .context("Failed to run tmux command")?;
    Ok((
        out.status.code().unwrap_or(1),
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
    ))
}

pub fn run_ok(args: &[&str]) {
    let _ = run(args);
}

pub fn get_opt(name: &str) -> Option<String> {
    match run(&["show-option", "-gv", &format!("@{name}")]) {
        Ok((0, out, _)) if !out.is_empty() => Some(out),
        _ => None,
    }
}

pub fn resolve_worktree_dir() -> String {
    get_opt("worktree-dir").unwrap_or_else(|| ".worktrees".to_string())
}

pub fn shell_command() -> String {
    if let Some(c) = get_opt("worktree-command") {
        return c;
    }
    let uid = Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();
    if !uid.is_empty() {
        if let Ok(out) = Command::new("getent").args(["passwd", &uid]).output() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if let Some(shell) = s.lines().next().and_then(|l| l.split(':').nth(6)) {
                    if !shell.is_empty() {
                        return shell.to_string();
                    }
                }
            }
        }
    }
    if let Ok(shell) = std::env::var("SHELL") {
        if !shell.is_empty() {
            return shell;
        }
    }
    "/bin/bash".to_string()
}

pub fn select_or_create_window(branch: &str, cwd: &str, command: &str) -> Result<()> {
    let win = format!("wt-{branch}");
    let exists = window_exists(&win)?;
    if exists {
        run_ok(&["select-window", "-t", &win]);
        run_ok(&["display-message", &format!("Resumed worktree: {branch}")]);
    } else {
        run_ok(&["new-window", "-n", &win, "-c", cwd, command]);
        run_ok(&["display-message", &format!("Created worktree: {branch}")]);
    }
    Ok(())
}

pub fn kill_window(branch: &str) -> Result<()> {
    let win = format!("wt-{branch}");
    if window_exists(&win)? {
        run_ok(&["kill-window", "-t", &win]);
    }
    Ok(())
}

fn window_exists(name: &str) -> Result<bool> {
    match run(&["list-windows", "-F", "#{window_name}"]) {
        Ok((0, out, _)) => Ok(out.lines().any(|l| l.trim() == name)),
        _ => Ok(false),
    }
}

pub fn show_error(msg: &str) -> Result<()> {
    eprintln!("{msg}");
    let tmp = std::env::temp_dir().join(format!("tmux-worktrees-err-{}.txt", std::process::id()));
    let _ = std::fs::write(&tmp, format!("{msg}\n"));
    let quoted = crate::shell_quote(tmp.to_string_lossy().as_ref());
    let cmd = format!(
        "cat {quoted}; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f {quoted}"
    );
    if run(&["display-popup", "-E", "-h", "20", &cmd]).is_ok() {
        return Ok(());
    }
    let _ = run(&["display-message", "-d", "5000", &format!("tmux-worktrees: {msg}")]);
    Ok(())
}
