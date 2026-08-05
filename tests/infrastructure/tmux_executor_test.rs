use std::io;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tmux_worktrees::infrastructure::command_runner::{CommandRunner, SystemCommandRunner};
use tmux_worktrees::infrastructure::tmux_executor::TmuxExecutor;
#[path = "../common/mod.rs"]
mod common;

use common::fakes::command_runner::FakeRunner;
use std::sync::Mutex;
use tmux_worktrees::core::error::Result;
use tmux_worktrees::utils::ShellQuoter;

// Isolated tmux test server — spawned once per test binary, shared by all
// integration tests.  Uses a unique socket so it never touches the user's
// real tmux server.

/// Wraps [`SystemCommandRunner`] and prepends `-L <socket>` to every `tmux`
/// invocation so all commands target the isolated test server.
struct SocketRunner {
    socket: String,
    inner: SystemCommandRunner,
}

impl CommandRunner for SocketRunner {
    fn run(&self, program: &str, args: &[&str], cwd: Option<&Path>) -> io::Result<Output> {
        if program == "tmux" {
            let mut full_args = vec!["-L", &self.socket];
            full_args.extend_from_slice(args);
            self.inner.run(program, &full_args, cwd)
        } else {
            self.inner.run(program, args, cwd)
        }
    }
}

/// Lazily-started test server socket name, or [`None`] if tmux is not
/// installed or could not be started.
static TEST_SOCKET: LazyLock<Option<String>> = LazyLock::new(|| {
    let socket = format!("tmux-worktrees-test-{}", std::process::id());
    // Kill any leftover server from a previous run with the same PID
    let _ = Command::new("tmux")
        .args(["-L", &socket, "kill-server"])
        .output();

    let result = Command::new("tmux")
        .args([
            "-L",
            &socket,
            "-f",
            "/dev/null",
            "new-session",
            "-d",
            "-s",
            "test",
        ])
        .output();

    match result {
        Ok(o) if o.status.success() => Some(socket),
        _ => None,
    }
});

fn ensure_test_server() -> Option<&'static str> {
    TEST_SOCKET.as_deref()
}

fn executor_for_test(socket: &str) -> TmuxExecutor {
    TmuxExecutor::with_runner(Box::new(SocketRunner {
        socket: socket.to_owned(),
        inner: SystemCommandRunner,
    }))
}

// Tests that don't need a server

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_executor_resolves_workspace_dir_default() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);
        let result = executor.resolve_workspace_dir();
        assert!(!result.is_empty());
        assert!(!result.contains('/'));
    }

    #[test]
    fn tmux_executor_resolves_shell_command_default() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);
        let result = executor.resolve_shell_command();
        assert!(!result.is_empty());
    }

    // Tests that target the isolated test server

    #[test]
    fn tmux_executor_can_run_command() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => {
                eprintln!("SKIP: tmux not available");
                return;
            }
        };
        let executor = executor_for_test(socket);
        let (status, stdout, _) = executor.run(&["display-message", "-p", "test"]).unwrap();
        assert_eq!(status, 0);
        assert_eq!(stdout.trim(), "test");
    }

    #[test]
    fn tmux_executor_can_get_option() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        executor.run_ok(&["set-option", "-g", "@test-opt", "test-value"]);

        let result = executor.get_option("test-opt");
        assert_eq!(result, Some("test-value".to_string()));

        executor.run_ok(&["set-option", "-gu", "@test-opt"]);
    }

    #[test]
    fn tmux_executor_can_list_windows() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);
        let (status, stdout, _) = executor
            .run(&["list-windows", "-F", "#{window_name}"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(!stdout.trim().is_empty());
    }

    #[test]
    fn tmux_executor_can_create_and_kill_window() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        executor.run_ok(&["new-window", "-n", "test-window", "-d", "sleep 10"]);

        let (status, stdout, _) = executor
            .run(&["list-windows", "-F", "#{window_name}"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.lines().any(|l| l.trim() == "test-window"));

        executor.run_ok(&["kill-window", "-t", "test-window"]);

        let (status, stdout, _) = executor
            .run(&["list-windows", "-F", "#{window_name}"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(!stdout.lines().any(|l| l.trim() == "test-window"));
    }

    #[test]
    fn tmux_executor_can_select_or_create_window() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        executor
            .select_or_create_window("test-branch", "/tmp", "bash")
            .unwrap();

        let (status, stdout, _) = executor
            .run(&["list-windows", "-F", "#{window_name}"])
            .unwrap();
        assert_eq!(status, 0);
        assert!(stdout.lines().any(|l| l.trim() == "ws-test-branch"));

        executor.kill_window("test-branch").unwrap();
    }

    #[test]
    fn tmux_executor_can_show_environment() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        executor.run_ok(&["set-environment", "-g", "TEST_VAR", "test-value"]);

        let result = executor.show_environment("TEST_VAR").unwrap();
        assert_eq!(result, Some("test-value".to_string()));

        executor.run_ok(&["set-environment", "-gu", "TEST_VAR"]);
    }

    #[test]
    #[ignore = "requires attached tmux client (display-popup needs a client to render)"]
    fn tmux_executor_can_display_popup() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        let result = executor.display_popup("50%", "50%", "/tmp", "echo hello");
        assert!(result.is_ok());
    }

    #[test]
    fn tmux_executor_can_get_current_pane_path() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };
        let executor = executor_for_test(socket);

        // With no client attached, `display-message -p #{pane_current_path}`
        // exits non-zero, so current_pane_path() exercises the fallback to
        // std::env::current_dir().  Both code paths are covered.
        let result = executor.current_pane_path().unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn isolated_server_is_truly_isolated() {
        let socket = match ensure_test_server() {
            Some(s) => s,
            None => return,
        };

        let marker = format!("isolated-{}", std::process::id());
        let executor = executor_for_test(socket);

        // 1. Set marker on the ISOLATED server
        executor.run_ok(&["set-environment", "-g", "ISOLATED_MARKER", &marker]);

        // 2. Read it back from the isolated server — MUST be present
        let (_, iso_stdout, _) = executor
            .run(&["show-environment", "-g", "ISOLATED_MARKER"])
            .unwrap();
        assert!(
            iso_stdout.contains(&marker),
            "marker not found on isolated server (socket={socket})"
        );

        // 3. Read it from the USER'S real server — MUST NOT be present
        let real_output = Command::new("tmux")
            .args(["show-environment", "-g", "ISOLATED_MARKER"])
            .output();
        if let Ok(out) = real_output {
            let real_env = String::from_utf8_lossy(&out.stdout);
            assert!(
                !real_env.contains(&marker),
                "isolated marker LEAKED onto user's real tmux server!\n\
                 real env: {real_env}"
            );
        }

        // Clean up
        executor.run_ok(&["set-environment", "-gu", "ISOLATED_MARKER"]);
    }
}

static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Build a `TmuxExecutor` backed by the given `FakeRunner`.
fn executor_with(runner: FakeRunner) -> TmuxExecutor {
    TmuxExecutor::with_runner(Box::new(runner))
}

#[test]
fn shell_quote_returns_unquoted_for_safe_chars() {
    assert_eq!(
        ShellQuoter::quote("hello-world_./:@%+,="),
        "hello-world_./:@%+,="
    );
}

#[test]
fn shell_quote_quotes_string_with_spaces() {
    assert_eq!(ShellQuoter::quote("hello world"), "'hello world'");
}

#[test]
fn shell_quote_escapes_single_quotes() {
    assert_eq!(ShellQuoter::quote("it's"), "'it'\\''s'");
}

#[test]
fn run_ok_never_panics_even_on_failure() {
    let f = FakeRunner::new();
    f.ok("tmux", "display-message:test", 1, "", "no server running");
    executor_with(f).run_ok(&["display-message", "test"]);
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
    assert_eq!(
        executor_with(f).resolve_shell_command(),
        "/usr/bin/fish".to_string()
    );
}

#[test]
fn resolve_shell_command_falls_through_to_getent() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let f = FakeRunner::new();
    f.ok(
        "tmux",
        "show-option:-gv:@worktree-command",
        1,
        "",
        "invalid option\n",
    );
    f.ok("id", "-u", 0, "1000\n", "");
    f.ok(
        "getent",
        "passwd:1000",
        0,
        "gbrennon:x:1000:1000:gbrennon:/home/gbrennon:/usr/bin/zsh\n",
        "",
    );
    let orig_shell = std::env::var("SHELL").ok();
    unsafe { std::env::remove_var("SHELL") };

    let result = executor_with(f).resolve_shell_command();
    assert_eq!(result, "/usr/bin/zsh");

    if let Some(s) = orig_shell {
        unsafe { std::env::set_var("SHELL", s) };
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
    f.err("id", "-u");
    let orig = std::env::var("SHELL").ok();
    unsafe { std::env::set_var("SHELL", "/bin/ksh") };
    assert_eq!(executor_with(f).resolve_shell_command(), "/bin/ksh");
    if let Some(s) = orig {
        unsafe { std::env::set_var("SHELL", s) };
    } else {
        unsafe { std::env::remove_var("SHELL") };
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
    unsafe { std::env::remove_var("SHELL") };
    assert_eq!(
        executor_with(f).resolve_shell_command(),
        "/bin/bash".to_string()
    );
    if let Some(s) = orig {
        unsafe { std::env::set_var("SHELL", s) };
    }
}

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

#[test]
fn select_or_create_window_creates_new_window() {
    let f = FakeRunner::new();
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
    executor_with(f).kill_window("nonexistent-window").unwrap();
}

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

#[test]
fn show_error_tries_popup_then_falls_back_to_display_message() {
    let f = FakeRunner::new();
    f.ok(
        "tmux",
        &display_popup_key("tmux-worktrees-err"),
        1,
        "",
        "no server",
    );
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
    let tmp = std::env::temp_dir()
        .join("tmux-worktrees-err-0.txt")
        .to_string_lossy()
        .to_string();
    let quoted = ShellQuoter::quote(&tmp);
    let cmd = format!(
        "cat {quoted}; echo; echo 'Press any key or wait 10s...'; read -t 10 -n1 2>/dev/null || true; rm -f {quoted}"
    );
    format!("display-popup:-E:-h:20:{cmd}")
}

fn display_msg_key(msg: &str) -> String {
    format!("display-message:-d:5000:{msg}")
}

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
