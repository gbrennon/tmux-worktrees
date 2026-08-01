use tmux_worktrees::infrastructure::tmux_executor::TmuxExecutor;
use std::process::Command;

#[cfg(test)]
mod tests {
    use super::*;

    fn tmux_available() -> bool {
        Command::new("tmux")
            .arg("-V")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn tmux_executor_resolves_workspace_dir_default() {
        let executor = TmuxExecutor;
        let result = executor.resolve_workspace_dir();
        assert!(!result.is_empty());
        assert!(!result.contains('/'));
    }

    #[test]
    fn tmux_executor_resolves_shell_command_default() {
        let executor = TmuxExecutor;
        let result = executor.resolve_shell_command();
        assert!(!result.is_empty());
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_run_command() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        let (status, stdout, _) = executor.run(&["display-message", "-p", "test"]).unwrap();
        assert_eq!(status, 0);
        assert_eq!(stdout.trim(), "test");
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_get_option() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        executor.run_ok(&["set-option", "-g", "@test-opt", "test-value"]);
        
        let result = executor.get_option("test-opt");
        assert_eq!(result, Some("test-value".to_string()));
        
        executor.run_ok(&["set-option", "-gu", "@test-opt"]);
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_list_windows() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        let (status, stdout, _) = executor.run(&["list-windows", "-F", "#{window_name}"]).unwrap();
        assert_eq!(status, 0);
        assert!(!stdout.trim().is_empty());
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_create_and_kill_window() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        executor.run_ok(&["new-window", "-n", "test-window", "-d", "sleep 10"]);
        
        let (status, stdout, _) = executor.run(&["list-windows", "-F", "#{window_name}"]).unwrap();
        assert_eq!(status, 0);
        assert!(stdout.lines().any(|l| l.trim() == "test-window"));
        
        executor.kill_window("test-window").unwrap();
        
        let (status, stdout, _) = executor.run(&["list-windows", "-F", "#{window_name}"]).unwrap();
        assert_eq!(status, 0);
        assert!(!stdout.lines().any(|l| l.trim() == "test-window"));
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_select_or_create_window() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        executor.select_or_create_window("test-branch", "/tmp", "bash").unwrap();
        
        let (status, stdout, _) = executor.run(&["list-windows", "-F", "#{window_name}"]).unwrap();
        assert_eq!(status, 0);
        assert!(stdout.lines().any(|l| l.trim() == "ws-test-branch"));
        
        executor.kill_window("test-branch").unwrap();
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_show_environment() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        executor.run_ok(&["set-environment", "-g", "TEST_VAR", "test-value"]);
        
        let result = executor.show_environment("TEST_VAR").unwrap();
        assert_eq!(result, Some("test-value".to_string()));
        
        executor.run_ok(&["set-environment", "-gu", "TEST_VAR"]);
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_display_popup() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        let result = executor.display_popup("50%", "50%", "/tmp", "echo hello");
        assert!(result.is_ok());
    }

    #[test]
    #[ignore = "requires running tmux server"]
    fn tmux_executor_can_get_current_pane_path() {
        if !tmux_available() {
            return;
        }
        let executor = TmuxExecutor;
        
        let result = executor.current_pane_path().unwrap();
        assert!(!result.is_empty());
    }
}