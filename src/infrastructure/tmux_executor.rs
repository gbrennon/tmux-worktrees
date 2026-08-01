use anyhow::{Context, Result};
use std::process::Command;

pub struct TmuxExecutor;

impl TmuxExecutor {
    pub fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
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

    pub fn run_ok(&self, args: &[&str]) {
        let _ = self.run(args);
    }

    pub fn get_option(&self, name: &str) -> Option<String> {
        match self.run(&["show-option", "-gv", &format!("@{name}")]) {
            Ok((0, out, _)) if !out.is_empty() => Some(out),
            _ => None,
        }
    }

    pub fn resolve_workspace_dir(&self) -> String {
        self.get_option("worktree-dir").unwrap_or_else(|| ".workspaces".to_string())
    }

    pub fn resolve_shell_command(&self) -> String {
        if let Some(c) = self.get_option("worktree-command") {
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

    pub fn select_or_create_window(&self, branch: &str, cwd: &str, command: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        let exists = self.window_exists(&win)?;
        if exists {
            self.run_ok(&["select-window", "-t", &win]);
            self.run_ok(&["display-message", &format!("Resumed workspace: {branch}")]);
        } else {
            self.run_ok(&["new-window", "-n", &win, "-c", cwd, command]);
            self.run_ok(&["display-message", &format!("Created workspace: {branch}")]);
        }
        Ok(())
    }

    pub fn kill_window(&self, branch: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        if self.window_exists(&win)? {
            self.run_ok(&["kill-window", "-t", &win]);
        }
        Ok(())
    }

    fn window_exists(&self, name: &str) -> Result<bool> {
        match self.run(&["list-windows", "-F", "#{window_name}"]) {
            Ok((0, out, _)) => Ok(out.lines().any(|l| l.trim() == name)),
            _ => Ok(false),
        }
    }

    pub fn show_error(&self, msg: &str) -> Result<()> {
        eprintln!("{msg}");
        let tmp = std::env::temp_dir().join(format!("tmux-worktrees-err-{}.txt", std::process::id()));
        let _ = std::fs::write(&tmp, format!("{msg}\n"));
        let quoted = shell_quote(tmp.to_string_lossy().as_ref());
        let cmd = format!(
            "cat {quoted}; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f {quoted}"
        );
        if self.run(&["display-popup", "-E", "-h", "20", &cmd]).is_ok() {
            return Ok(());
        }
        let _ = self.run(&["display-message", "-d", "5000", &format!("tmux-worktrees: {msg}")]);
        Ok(())
    }

    pub fn display_popup(&self, width: &str, height: &str, dir: &str, cmd: &str) -> Result<()> {
        self.run(&["display-popup", "-E", "-w", width, "-h", height, "-d", dir, cmd])?;
        Ok(())
    }

    pub fn current_pane_path(&self) -> Result<String> {
        match self.run(&["display-message", "-p", "#{pane_current_path}"]) {
            Ok((0, out, _)) => Ok(out.trim().to_string()),
            _ => std::env::current_dir()
                .map(|p| p.to_string_lossy().into_owned())
                .map_err(|e| anyhow::anyhow!(e)),
        }
    }

    pub fn show_environment(&self, name: &str) -> Result<Option<String>> {
        match self.run(&["show-environment", "-g", name]) {
            Ok((0, out, _)) if out.starts_with(&format!("{}=", name)) => {
                Ok(out.strip_prefix(&format!("{}=", name)).map(|s| s.trim().to_string()))
            }
            _ => Ok(None),
        }
    }
}

fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || "_-./:@%+,=".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_executor_resolves_workspace_dir_default() {
        let executor = TmuxExecutor;
        // This may pick up user's tmux config, so just verify it returns a valid directory name
        let result = executor.resolve_workspace_dir();
        assert!(!result.is_empty());
        assert!(!result.contains('/'));
    }
}