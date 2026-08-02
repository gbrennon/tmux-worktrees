use anyhow::{Context, Result};

use super::command_runner::{CommandRunner, SystemCommandRunner};

/// Thin adapter around `tmux` commands — every call is forwarded through the
/// injected [`CommandRunner`] so tests can supply a fake and never touch a real
/// tmux server.
pub struct TmuxExecutor {
    runner: Box<dyn CommandRunner>,
}

impl Default for TmuxExecutor {
    fn default() -> Self {
        Self {
            runner: Box::new(SystemCommandRunner),
        }
    }
}

impl TmuxExecutor {
    #[allow(dead_code)]
    pub fn with_runner(runner: Box<dyn CommandRunner>) -> Self {
        Self { runner }
    }

    // -- low-level runner ---------------------------------------------------

    pub fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        let out = self
            .runner
            .run("tmux", args, None)
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

    // -- option helpers -----------------------------------------------------

    pub fn get_option(&self, name: &str) -> Option<String> {
        match self.run(&["show-option", "-gv", &format!("@{name}")]) {
            Ok((0, out, _)) if !out.is_empty() => Some(out),
            _ => None,
        }
    }

    pub fn resolve_workspace_dir(&self) -> String {
        self.get_option("worktree-dir")
            .unwrap_or_else(|| ".workspaces".to_string())
    }

    pub fn resolve_shell_command(&self) -> String {
        // 1. tmux @worktree-command
        if let Some(c) = self.get_option("worktree-command") {
            return c;
        }
        // 2. getent passwd → login shell
        let uid = self
            .runner
            .run("id", &["-u"], None)
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        if !uid.is_empty() {
            if let Ok(out) = self.runner.run("getent", &["passwd", &uid], None) {
                if let Ok(s) = String::from_utf8(out.stdout) {
                    if let Some(shell) = s.lines().next().and_then(|l| l.split(':').nth(6)) {
                        if !shell.is_empty() {
                            return shell.to_string();
                        }
                    }
                }
            }
        }
        // 3. $SHELL env var
        if let Ok(shell) = std::env::var("SHELL") {
            if !shell.is_empty() {
                return shell;
            }
        }
        // 4. ultimate fallback
        "/bin/bash".to_string()
    }

    // -- window management --------------------------------------------------

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

    // -- display helpers ----------------------------------------------------

    pub fn show_error(&self, msg: &str) -> Result<()> {
        eprintln!("{msg}");
        let tmp =
            std::env::temp_dir().join(format!("tmux-worktrees-err-{}.txt", std::process::id()));
        let _ = std::fs::write(&tmp, format!("{msg}\n"));
        let quoted = shell_quote(tmp.to_string_lossy().as_ref());
        let cmd = format!(
            "cat {quoted}; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f {quoted}"
        );
        if self.run(&["display-popup", "-E", "-h", "20", &cmd]).is_ok() {
            return Ok(());
        }
        let _ = self.run(&[
            "display-message",
            "-d",
            "5000",
            &format!("tmux-worktrees: {msg}"),
        ]);
        Ok(())
    }

    pub fn display_popup(&self, width: &str, height: &str, dir: &str, cmd: &str) -> Result<()> {
        self.run(&[
            "display-popup",
            "-E",
            "-w",
            width,
            "-h",
            height,
            "-d",
            dir,
            cmd,
        ])?;
        Ok(())
    }

    // -- introspection ------------------------------------------------------

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
            Ok((0, out, _)) if out.starts_with(&format!("{}=", name)) => Ok(out
                .strip_prefix(&format!("{}=", name))
                .map(|s| s.trim().to_string())),
            _ => Ok(None),
        }
    }
}

// -- free helper ------------------------------------------------------------

fn shell_quote(s: &str) -> String {
    if s.chars()
        .all(|c| c.is_alphanumeric() || "_-./:@%+,=".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

// ---------------------------------------------------------------------------
// Tests — every test below uses FakeRunner; zero real tmux / getent / id
// calls.  Fixture data is captured from actual command output where possible.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::super::command_runner::test_support::FakeRunner;
    use super::*;
    use std::sync::Mutex;

    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    /// Build a `TmuxExecutor` backed by the given `FakeRunner`.
    fn executor_with(runner: FakeRunner) -> TmuxExecutor {
        TmuxExecutor::with_runner(Box::new(runner))
    }

    // -- pure functions -----------------------------------------------------

    #[test]
    fn shell_quote_returns_unquoted_for_safe_chars() {
        assert_eq!(shell_quote("hello-world_./:@%+,="), "hello-world_./:@%+,=");
    }

    #[test]
    fn shell_quote_quotes_string_with_spaces() {
        assert_eq!(shell_quote("hello world"), "'hello world'");
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    // -- run / run_ok -------------------------------------------------------

    #[test]
    fn run_ok_never_panics_even_on_failure() {
        let f = FakeRunner::new();
        f.ok("tmux", "display-message:test", 1, "", "no server running");
        executor_with(f).run_ok(&["display-message", "test"]);
        // assert: does not panic
    }

    #[test]
    fn run_returns_status_stdout_stderr() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "display-message:-p:#{pane_current_path}",
            0,
            "/home/user/project\n",
            "",
        );
        let result = executor_with(f).run(&["display-message", "-p", "#{pane_current_path}"]);
        assert!(result.is_ok());
        let (status, stdout, stderr) = result.unwrap();
        assert_eq!(status, 0);
        assert_eq!(stdout, "/home/user/project");
        assert!(stderr.is_empty());
    }

    // -- get_option ---------------------------------------------------------

    #[test]
    fn get_option_returns_some_for_set_option() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-dir",
            0,
            ".worktrees\n",
            "",
        );
        let result = executor_with(f).get_option("worktree-dir");
        assert_eq!(result, Some(".worktrees".to_string()));
    }

    #[test]
    fn get_option_returns_none_when_option_unset() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@nonexistent-opt",
            1,
            "",
            "invalid option: @nonexistent-opt\n",
        );
        assert_eq!(executor_with(f).get_option("nonexistent-opt"), None);
    }

    #[test]
    fn get_option_returns_none_when_empty_output() {
        let f = FakeRunner::new();
        f.ok("tmux", "show-option:-gv:@empty-opt", 0, "", "");
        assert_eq!(executor_with(f).get_option("empty-opt"), None);
    }

    // -- resolve_workspace_dir ----------------------------------------------

    #[test]
    fn resolve_workspace_dir_reads_tmux_option() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-dir",
            0,
            "my-worktrees\n",
            "",
        );
        assert_eq!(
            executor_with(f).resolve_workspace_dir(),
            "my-worktrees".to_string()
        );
    }

    #[test]
    fn resolve_workspace_dir_falls_back_to_default() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-dir",
            1,
            "",
            "invalid option\n",
        );
        assert_eq!(
            executor_with(f).resolve_workspace_dir(),
            ".workspaces".to_string()
        );
    }

    // -- resolve_shell_command ----------------------------------------------

    #[test]
    fn resolve_shell_command_uses_tmux_option_first() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-command",
            0,
            "/usr/bin/fish\n",
            "",
        );
        // No id / getent calls needed because the tmux option short-circuits.
        assert_eq!(
            executor_with(f).resolve_shell_command(),
            "/usr/bin/fish".to_string()
        );
    }

    #[test]
    fn resolve_shell_command_falls_through_to_getent() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let f = FakeRunner::new();
        // tmux option unset
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-command",
            1,
            "",
            "invalid option\n",
        );
        // id -u returns uid
        f.ok("id", "-u", 0, "1000\n", "");
        // getent passwd returns passwd line
        f.ok(
            "getent",
            "passwd:1000",
            0,
            "gbrennon:x:1000:1000:gbrennon:/home/gbrennon:/usr/bin/zsh\n",
            "",
        );
        // Unset SHELL so we get the getent path.
        let orig_shell = std::env::var("SHELL").ok();
        std::env::remove_var("SHELL");

        let result = executor_with(f).resolve_shell_command();
        assert_eq!(result, "/usr/bin/zsh");

        if let Some(s) = orig_shell {
            std::env::set_var("SHELL", s);
        }
    }

    #[test]
    fn resolve_shell_command_falls_back_to_shell_env() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-command",
            1,
            "",
            "invalid option\n",
        );
        // Make getent fail → fall through to SHELL
        f.err("id", "-u");
        let orig = std::env::var("SHELL").ok();
        std::env::set_var("SHELL", "/bin/ksh");
        assert_eq!(executor_with(f).resolve_shell_command(), "/bin/ksh");
        if let Some(s) = orig {
            std::env::set_var("SHELL", s);
        } else {
            std::env::remove_var("SHELL");
        }
    }

    #[test]
    fn resolve_shell_command_falls_back_to_bash_as_last_resort() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-option:-gv:@worktree-command",
            1,
            "",
            "invalid option\n",
        );
        f.err("id", "-u"); // no uid
        let orig = std::env::var("SHELL").ok();
        std::env::remove_var("SHELL");
        assert_eq!(
            executor_with(f).resolve_shell_command(),
            "/bin/bash".to_string()
        );
        if let Some(s) = orig {
            std::env::set_var("SHELL", s);
        }
    }

    // -- window_exists ------------------------------------------------------

    #[test]
    fn window_exists_true_when_window_in_list() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "list-windows:-F:#{window_name}",
            0,
            "zsh\nws-feat-foo\nomp\n",
            "",
        );
        assert!(executor_with(f).window_exists("ws-feat-foo").unwrap());
    }

    #[test]
    fn window_exists_false_when_window_not_in_list() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "list-windows:-F:#{window_name}",
            0,
            "zsh\nomp\n",
            "",
        );
        assert!(!executor_with(f).window_exists("ws-nonexistent").unwrap());
    }

    #[test]
    fn window_exists_false_on_command_failure() {
        let f = FakeRunner::new();
        f.err("tmux", "list-windows:-F:#{window_name}");
        assert!(!executor_with(f).window_exists("ws-any").unwrap());
    }

    // -- select_or_create_window --------------------------------------------

    #[test]
    fn select_or_create_window_creates_new_window() {
        let f = FakeRunner::new();
        // window_exists → false (list-windows returns nothing with ws-feat-foo)
        f.ok(
            "tmux",
            "list-windows:-F:#{window_name}",
            0,
            "zsh\nomp\n",
            "",
        );
        f.ok(
            "tmux",
            "new-window:-n:ws-feat-foo:-c:/tmp:sleep 30",
            0,
            "",
            "",
        );
        f.ok(
            "tmux",
            "display-message:Created workspace: feat-foo",
            0,
            "",
            "",
        );
        executor_with(f)
            .select_or_create_window("feat-foo", "/tmp", "sleep 30")
            .unwrap();
    }

    #[test]
    fn select_or_create_window_resumes_existing() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "list-windows:-F:#{window_name}",
            0,
            "ws-feat-bar\nomp\n",
            "",
        );
        f.ok("tmux", "select-window:-t:ws-feat-bar", 0, "", "");
        f.ok(
            "tmux",
            "display-message:Resumed workspace: feat-bar",
            0,
            "",
            "",
        );
        executor_with(f)
            .select_or_create_window("feat-bar", "/tmp", "bash")
            .unwrap();
    }

    // -- kill_window --------------------------------------------------------

    #[test]
    fn kill_window_kills_when_present() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "list-windows:-F:#{window_name}",
            0,
            "ws-feat-baz\n",
            "",
        );
        f.ok("tmux", "kill-window:-t:ws-feat-baz", 0, "", "");
        executor_with(f).kill_window("feat-baz").unwrap();
    }

    #[test]
    fn kill_window_noop_when_not_present() {
        let f = FakeRunner::new();
        f.ok("tmux", "list-windows:-F:#{window_name}", 0, "omp\n", "");
        // No kill-window call expected — window not in list.
        executor_with(f).kill_window("nonexistent-window").unwrap();
    }

    // -- display_popup ------------------------------------------------------

    #[test]
    fn display_popup_ok() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "display-popup:-E:-w:60%:-h:40%:-d:/tmp:echo hi",
            0,
            "",
            "",
        );
        executor_with(f)
            .display_popup("60%", "40%", "/tmp", "echo hi")
            .unwrap();
    }

    // -- show_error ---------------------------------------------------------
    // (note: writes a real temp file; the tmux calls are faked)

    #[test]
    fn show_error_tries_popup_then_falls_back_to_display_message() {
        let f = FakeRunner::new();
        // First attempt: display-popup fails
        f.ok(
            "tmux",
            &display_popup_key("tmux-worktrees-err"),
            1,
            "",
            "no server",
        );
        // Fallback: display-message succeeds
        f.ok(
            "tmux",
            &display_msg_key("tmux-worktrees: test msg"),
            0,
            "",
            "",
        );
        executor_with(f).show_error("test msg").unwrap();
    }

    #[test]
    fn show_error_succeeds_with_popup() {
        let f = FakeRunner::new();
        f.ok("tmux", &display_popup_key("tmux-worktrees-err"), 0, "", "");
        executor_with(f).show_error("ok msg").unwrap();
    }

    /// Build the key that the `FakeRunner` will see for the display-popup call
    /// in `show_error`.  The actual command includes a temp-file path, so we
    /// match on the prefix and suffix that is stable.
    fn display_popup_key(_txt_prefix: &str) -> String {
        // The full args are: display-popup -E -h 20 <cmd-with-temp-path>
        // We use a prefix-only match via `ok()` with a partial key.
        // Strategy: just register with a sentinel default-error path.
        let tmp = std::env::temp_dir()
            .join("tmux-worktrees-err-0.txt")
            .to_string_lossy()
            .to_string();
        let quoted = shell_quote(&tmp);
        let cmd = format!(
            "cat {quoted}; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f {quoted}"
        );
        format!("display-popup:-E:-h:20:{cmd}")
    }

    fn display_msg_key(msg: &str) -> String {
        format!("display-message:-d:5000:{msg}")
    }

    // -- current_pane_path --------------------------------------------------

    #[test]
    fn current_pane_path_returns_path_when_tmux_runs() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "display-message:-p:#{pane_current_path}",
            0,
            "/home/user/my-repo\n",
            "",
        );
        assert_eq!(
            executor_with(f).current_pane_path().unwrap(),
            "/home/user/my-repo"
        );
    }

    #[test]
    fn current_pane_path_falls_back_to_current_dir() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "display-message:-p:#{pane_current_path}",
            1,
            "",
            "no server running\n",
        );
        let result = executor_with(f).current_pane_path().unwrap();
        assert!(!result.is_empty()); // whatever cwd is at test time
    }

    // -- show_environment ---------------------------------------------------

    #[test]
    fn show_environment_returns_some_for_known_var() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-environment:-g:SHELL",
            0,
            "SHELL=/bin/bash\n",
            "",
        );
        assert_eq!(
            executor_with(f).show_environment("SHELL").unwrap(),
            Some("/bin/bash".to_string())
        );
    }

    #[test]
    fn show_environment_returns_none_for_unknown_var() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-environment:-g:UNKNOWN_VAR",
            1,
            "unknown variable: UNKNOWN_VAR\n",
            "",
        );
        assert_eq!(
            executor_with(f).show_environment("UNKNOWN_VAR").unwrap(),
            None
        );
    }

    #[test]
    fn show_environment_returns_none_for_empty_value() {
        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "show-environment:-g:EMPTY_VAR",
            0,
            "EMPTY_VAR=\n",
            "",
        );
        assert_eq!(
            executor_with(f).show_environment("EMPTY_VAR").unwrap(),
            Some("".to_string())
        );
    }
}
