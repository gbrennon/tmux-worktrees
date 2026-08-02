use std::io;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::LazyLock;

use tmux_worktrees::infrastructure::command_runner::{CommandRunner, SystemCommandRunner};
use tmux_worktrees::infrastructure::tmux_executor::TmuxExecutor;

// ---------------------------------------------------------------------------
// Isolated tmux test server — spawned once per test binary, shared by all
// integration tests.  Uses a unique socket so it never touches the user's
// real tmux server.
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Tests that don't need a server
// ---------------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Tests that target the isolated test server
    // -----------------------------------------------------------------------

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
