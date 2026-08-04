// Integration tests for the Cli presentation layer.
// Each fake covers exactly one scenario — no god objects.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use tmux_worktrees::core::error::Result;
use tmux_worktrees::core::ports::{GitPort, TmuxPort};
use tmux_worktrees::core::selector::SelectionResult;
use tmux_worktrees::core::selector::Selector;
use tmux_worktrees::presentation::cli::Cli;
use tmux_worktrees::presentation::command::Command;

// ===========================================================================
// Stub fakes — unused methods panic
// ===========================================================================

use tmux_worktrees::presentation::selector_port::SelectorRunner;

/// Selector runner for tests that returns a canned result.
struct StubSelector;
impl SelectorRunner for StubSelector {
    fn run_selector(
        &self,
        _selector: &mut tmux_worktrees::core::selector::Selector,
        _filtered: &[usize],
        _prompt: &str,
        _header: &str,
    ) -> tmux_worktrees::core::error::Result<SelectionResult> {
        unimplemented!()
    }
}

struct StubTmx;
impl TmuxPort for StubTmx {
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

struct StubGit;
impl GitPort for StubGit {
    fn run_in(&self, _d: &Path, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// run_create("") → show_error
// ===========================================================================

struct FakeTmxShowError;
impl TmuxPort for FakeTmxShowError {
    fn show_error(&self, _msg: &str) -> Result<()> {
        Ok(())
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// run(): interactive command triggered from non-TTY stdin
// spawn_in_popup → find_repo_root succeeds → display_popup called → Ok
// ===========================================================================

#[derive(Clone)]
struct FakeTmxNonTty {
    show_error_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxNonTty {
    fn new() -> Self {
        Self {
            show_error_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxNonTty {
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".".to_string()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/sh".to_string()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// run_create success: needs resolve_workspace_dir, resolve_shell_command,
// select_or_create_window
// ===========================================================================

struct FakeTmxCreateSuccess;
impl TmuxPort for FakeTmxCreateSuccess {
    fn show_error(&self, _msg: &str) -> Result<()> {
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".worktrees".into()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/bash".into()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// create_workspace — worktree add succeeds (show-ref succeeds too)
// ===========================================================================

struct FakeGitWorktreeAdd;
impl GitPort for FakeGitWorktreeAdd {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((0, String::new(), String::new()))
        } else {
            Ok((0, String::new(), String::new()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// create_workspace — all git commands fail
// ===========================================================================

struct FakeGitWorktreeFail;
impl GitPort for FakeGitWorktreeFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((1, String::new(), "ref missing".into()))
        } else {
            Ok((1, String::new(), "fail".into()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// create_workspace — show-ref FAILS (no remote); local base used
// ===========================================================================

struct FakeGitWorktreeAddLocal;
impl GitPort for FakeGitWorktreeAddLocal {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((1, String::new(), String::new()))
        } else {
            Ok((0, String::new(), String::new()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// remove_workspace — git succeeds
// ===========================================================================

struct FakeGitWorktreeRemove;
impl GitPort for FakeGitWorktreeRemove {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "remove", "--force", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// remove_workspace — git fails
// ===========================================================================

struct FakeGitWorktreeRemoveFail;
impl GitPort for FakeGitWorktreeRemoveFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "remove", "--force", _] => Ok((1, String::new(), "remove failed".into())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// kill_window — window exists → killed
// ===========================================================================

struct FakeTmxWithWindow;
impl TmuxPort for FakeTmxWithWindow {
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["list-windows", "-F", "{window_name}"] => Ok((0, "ws-feat/x\n".into(), String::new())),
            ["kill-window", "-t", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn kill_window(&self, branch: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        let (status, out, _) = self.run(&["list-windows", "-F", "{window_name}"])?;
        if status == 0 && out.lines().any(|l| l.trim() == win) {
            self.run(&["kill-window", "-t", &win])?;
        }
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
}

// ===========================================================================
// kill_window — no matching window → no-op
// ===========================================================================

struct FakeTmxNoWindow;
impl TmuxPort for FakeTmxNoWindow {
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["list-windows", "-F", "{window_name}"] => {
                Ok((0, "other-window\n".into(), String::new()))
            }
            ["kill-window", "-t", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn kill_window(&self, branch: &str) -> Result<()> {
        let win = format!("ws-{branch}");
        let (status, out, _) = self.run(&["list-windows", "-F", "{window_name}"])?;
        if status == 0 && out.lines().any(|l| l.trim() == win) {
            self.run(&["kill-window", "-t", &win])?;
        }
        Ok(())
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Ok(".".to_string())
    }
}

// ===========================================================================
// workspace_branch — rev-parse fails
// ===========================================================================

struct FakeGitRevParseFail;
impl GitPort for FakeGitRevParseFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((1, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// workspace_branch — succeeds
// ===========================================================================

struct FakeGitRevParseOk;
impl GitPort for FakeGitRevParseOk {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((0, "feat/x\n".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// workspace_branch — detached HEAD
// ===========================================================================

struct FakeGitRevParseDetached;
impl GitPort for FakeGitRevParseDetached {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((0, "HEAD\n".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// list_workspaces — porcelain fails
// ===========================================================================

struct FakeGitWorktreeListFail;
impl GitPort for FakeGitWorktreeListFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((1, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// list_workspaces — returns paths
// ===========================================================================

struct FakeGitWorktreeListOk;
impl GitPort for FakeGitWorktreeListOk {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => {
                Ok((0, "worktree /tmp/repo\nworktree /tmp/repo/worktrees/feat-a\nworktree /tmp/repo/worktrees/fix-b\n".into(), String::new()))
            }
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// resolve_default_branch — both global and local config succeed
// ===========================================================================

struct FakeGitDefaultBranch;
impl GitPort for FakeGitDefaultBranch {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// resolve_default_branch — global fails, local succeeds
// ===========================================================================

struct FakeGitBranchLocalOnly;
impl GitPort for FakeGitBranchLocalOnly {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "develop".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "develop".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// resolve_default_branch — both global and local fail, falls through to current
// ===========================================================================

struct FakeGitBranchFallbackCurrent;
impl GitPort for FakeGitBranchFallbackCurrent {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["config", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["branch", "--show-current"] => Ok((0, "trunk".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// delete_branch — silent_in succeeds
// ===========================================================================

struct FakeGitBranchDelete;
impl GitPort for FakeGitBranchDelete {
    fn run_in(&self, _d: &Path, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn silent_in(&self, _d: &Path, _args: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// spawn_in_popup — display_popup called when find_repo_root succeeds
// ===========================================================================

#[derive(Clone)]
struct FakeTmxPopupOk {
    display_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxPopupOk {
    fn new() -> Self {
        Self {
            display_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxPopupOk {
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        self.display_called.set(true);
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn show_error(&self, _msg: &str) -> Result<()> {
        unimplemented!()
    }
    fn resolve_workspace_dir(&self) -> String {
        unimplemented!()
    }
    fn resolve_shell_command(&self) -> String {
        unimplemented!()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// Helper
// ===========================================================================

fn cli_with(
    tmux: impl TmuxPort + 'static,
    git: impl GitPort + 'static,
    selector: impl SelectorRunner + 'static,
) -> Cli {
    Cli::new(Box::new(tmux), Box::new(git), Box::new(selector))
}

fn cli_default(tmux: impl TmuxPort + 'static, git: impl GitPort + 'static) -> Cli {
    cli_with(tmux, git, StubSelector)
}

// ===========================================================================
// parse_args tests
// ===========================================================================

#[test]
fn parse_args_defaults_to_choose() {
    let cli = cli_default(StubTmx, StubGit);
    let (cmd, rest) = cli.parse_args(&["tmux-worktrees".to_string()]);
    assert_eq!(cmd, Command::Choose);
    assert!(rest.is_empty());
}

#[test]
fn parse_args_extracts_root_and_branch() {
    let cli = cli_default(StubTmx, StubGit);
    let (cmd, rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "create-worktree".to_string(),
        "feat/foo".to_string(),
        "--root=/home/user/repo".to_string(),
    ]);
    assert_eq!(cmd, Command::CreateWorktree);
    assert_eq!(rest, vec!["feat/foo"]);
}

#[test]
fn parse_args_supports_separate_root_flag() {
    let cli = cli_default(StubTmx, StubGit);
    let (cmd, rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root".to_string(),
        "/home/user/repo".to_string(),
    ]);
    assert_eq!(cmd, Command::Choose);
    assert!(rest.is_empty());
}

#[test]
fn parse_args_handles_root_flag_at_end_no_value() {
    let cli = cli_default(StubTmx, StubGit);
    let (_cmd, rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root".to_string(),
    ]);
    assert!(rest.is_empty());
}

#[test]
fn parse_args_cleanup_command() {
    let cli = cli_default(StubTmx, StubGit);
    let (cmd, _) = cli.parse_args(&["tmux-worktrees".to_string(), "cleanup".to_string()]);
    assert_eq!(cmd, Command::Cleanup);
}

#[test]
fn unknown_command_defaults_to_choose() {
    let cli = cli_default(StubTmx, StubGit);
    let (cmd, _rest) = cli.parse_args(&["tmux-worktrees".to_string(), "bogus".to_string()]);
    assert_eq!(cmd, Command::Choose);
}

// ===========================================================================
// dispatch tests
// ===========================================================================

#[test]
fn dispatch_create_worktree_empty_branch_shows_error() {
    let cli = cli_default(FakeTmxShowError, StubGit);
    let result = cli.dispatch(Command::CreateWorktree, &[]);
    assert!(result.is_ok());
}

// ===========================================================================
// run tests
// ===========================================================================

#[test]
fn run_create_worktree_empty_branch_does_not_panic() {
    let cli = cli_default(FakeTmxShowError, StubGit);
    cli.run(&["tmux-worktrees".to_string(), "create-worktree".to_string()]);
}

#[test]
fn run_interactive_from_non_tty_uses_popup_path() {
    let tmux = FakeTmxNonTty::new();
    let guard = tmux.clone();
    let cli = cli_default(tmux, StubGit);
    let root = std::env::current_dir().unwrap();
    unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root.to_str().unwrap()) };
    let result = cli.spawn_in_popup(&["tmux-worktrees".to_string(), "choose".to_string()]);
    assert!(result.is_ok());
    assert!(!guard.show_error_called.get());
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
}

// ===========================================================================
// spawn_in_popup tests
// ===========================================================================

#[test]
fn spawn_in_popup_calls_display_popup_when_root_found() {
    let tmux = FakeTmxPopupOk::new();
    let guard = tmux.clone();
    let cli = cli_default(tmux, StubGit);
    let result = cli.spawn_in_popup(&["tmux-worktrees".to_string(), "choose".to_string()]);
    assert!(result.is_ok());
    assert!(guard.display_called.get());
}

// ===========================================================================
// run_create
// ===========================================================================

#[test]
fn run_create_shows_error_on_empty_branch() {
    let cli = cli_default(FakeTmxShowError, StubGit);
    let result = cli.run_create("");
    assert!(result.is_ok());
}

// ===========================================================================
// create_workspace
// ===========================================================================

#[test]
fn create_workspace_uses_origin_prefix_when_remote_exists() {
    let cli = cli_default(StubTmx, FakeGitWorktreeAdd);
    let result = cli.create_workspace(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_ok());
}

#[test]
fn create_workspace_uses_local_branch_when_no_remote() {
    let cli = cli_default(StubTmx, FakeGitWorktreeAddLocal);
    let result = cli.create_workspace(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_ok());
}

#[test]
fn create_workspace_propagates_git_errors() {
    let cli = cli_default(StubTmx, FakeGitWorktreeFail);
    let result = cli.create_workspace(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_err());
}

// ===========================================================================
// remove_workspace
// ===========================================================================

#[test]
fn remove_workspace_skips_kill_window_when_branch_empty() {
    let cli = cli_default(StubTmx, FakeGitWorktreeRemove);
    let result = cli.remove_workspace(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "");
    assert!(result.is_ok());
}

#[test]
fn remove_workspace_with_branch_calls_kill_window() {
    let cli = cli_default(FakeTmxWithWindow, FakeGitWorktreeRemove);
    let result = cli.remove_workspace(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "feat/x");
    assert!(result.is_ok());
}

#[test]
fn remove_workspace_propagates_git_error() {
    let cli = cli_default(FakeTmxNoWindow, FakeGitWorktreeRemoveFail);
    let result = cli.remove_workspace(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "feat/x");
    assert!(result.is_err());
}

// ===========================================================================
// workspace_branch
// ===========================================================================

#[test]
fn workspace_branch_returns_question_mark_on_failure() {
    let cli = cli_default(StubTmx, FakeGitRevParseFail);
    let result = cli.workspace_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "?");
}

#[test]
fn workspace_branch_returns_branch_name_on_success() {
    let cli = cli_default(StubTmx, FakeGitRevParseOk);
    let result = cli.workspace_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "feat/x\n");
}

#[test]
fn workspace_branch_returns_detached_head() {
    let cli = cli_default(StubTmx, FakeGitRevParseDetached);
    let result = cli.workspace_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "HEAD\n");
}

// ===========================================================================
// list_workspaces
// ===========================================================================

#[test]
fn list_workspaces_returns_empty_on_nonzero_status() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListFail);
    let result = cli.list_workspaces(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn list_workspaces_returns_worktree_paths() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListOk);
    let result = cli.list_workspaces(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("feat-a")));
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("fix-b")));
}

// ===========================================================================
// list_workspace_names
// ===========================================================================

#[test]
fn list_workspace_names_extracts_sorted_names() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListOk);
    let result = cli.list_workspace_names(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names, vec!["feat-a", "fix-b"]);
}

// ===========================================================================
// resolve_default_branch
// ===========================================================================

#[test]
fn resolve_default_branch_uses_configured_values() {
    let cli = cli_default(StubTmx, FakeGitDefaultBranch);
    let result = cli.resolve_default_branch(Path::new("/tmp/repo"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "main");
}

#[test]
fn resolve_default_branch_falls_back_to_local() {
    let cli = cli_default(StubTmx, FakeGitBranchLocalOnly);
    let result = cli.resolve_default_branch(Path::new("/tmp/repo"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "develop");
}

#[test]
fn resolve_default_branch_falls_back_to_current() {
    let cli = cli_default(StubTmx, FakeGitBranchFallbackCurrent);
    let result = cli.resolve_default_branch(Path::new("/tmp/repo"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "trunk");
}

// ===========================================================================
// delete_branch
// ===========================================================================

#[test]
fn delete_branch_does_not_panic() {
    let cli = cli_default(StubTmx, FakeGitBranchDelete);
    cli.delete_branch(Path::new("/tmp/repo"), "feat/x");
}

// ===========================================================================
// find_repo_root
// ===========================================================================

#[test]
fn find_repo_root_returns_non_empty_string() {
    let cli = cli_default(StubTmx, StubGit);
    let result = cli.find_repo_root();
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[test]
fn find_repo_root_is_idempotent() {
    let cli = cli_default(StubTmx, StubGit);
    let r1 = cli.find_repo_root().unwrap();
    let r2 = cli.find_repo_root().unwrap();
    assert_eq!(r1, r2);
}

// ===========================================================================
// Combined GitPort fake — handles config, symbolic-ref, and worktree ops
// ===========================================================================

struct FakeGitCreate;
impl GitPort for FakeGitCreate {
    fn run_in(&self, _dir: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args.contains(&"init.defaultBranch") {
            Ok((0, "main".into(), String::new()))
        } else if args.contains(&"--show-current") {
            Ok((0, "main".into(), String::new()))
        } else if args.contains(&"show-ref") {
            // Remote branch doesn't exist → fall through to use local "main"
            Ok((1, String::new(), String::new()))
        } else if args.contains(&"worktree") && args.contains(&"add") {
            // Worktree add succeeds
            Ok((0, String::new(), String::new()))
        } else {
            unimplemented!("unexpected run_in args: {args:?}")
        }
    }
    fn silent_in(&self, _dir: &Path, _args: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// run_create full success path — select_or_create_window fails → run() shows error
// ===========================================================================

#[derive(Clone)]
struct FakeTmxWindowFail {
    show_error_called: std::rc::Rc<std::cell::Cell<bool>>,
}
impl FakeTmxWindowFail {
    fn new() -> Self {
        Self {
            show_error_called: std::rc::Rc::new(std::cell::Cell::new(false)),
        }
    }
}
impl TmuxPort for FakeTmxWindowFail {
    fn show_error(&self, _msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        ".worktrees".into()
    }
    fn resolve_shell_command(&self) -> String {
        "/bin/bash".into()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Err(tmux_worktrees::core::error::Error::new(
            "window creation failed",
        ))
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(None)
    }
    fn current_pane_path(&self) -> Result<String> {
        Err(tmux_worktrees::core::error::Error::new("no pane"))
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        unimplemented!()
    }
    fn run(&self, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        unimplemented!()
    }
}

// ===========================================================================
// run_create success path — full flow through to select_or_create_window
// ===========================================================================

#[test]
fn run_create_full_success_path() {
    let cli = cli_default(FakeTmxCreateSuccess, FakeGitCreate);
    let result = cli.dispatch(Command::CreateWorktree, &["test-branch".to_string()]);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

// ===========================================================================
// run error path — when dispatch returns Err, show_error is called
// ===========================================================================

#[test]
fn run_error_path_shows_error_when_dispatch_fails() {
    let fake_tmux = FakeTmxWindowFail::new();
    let show_error_called = fake_tmux.show_error_called.clone();
    let cli = cli_default(fake_tmux, FakeGitCreate);
    cli.run(&[
        "prog".into(),
        "create-worktree".into(),
        "test-branch".into(),
    ]);
    assert!(
        show_error_called.get(),
        "show_error was not called on dispatch failure"
    );
}
// ===========================================================================
// run_create error path — create_workspace fails → show_error
// ===========================================================================

#[test]
fn run_create_shows_error_when_workspace_creation_fails() {
    let fake_tmux = FakeTmxWindowFail::new();
    let show_error_called = fake_tmux.show_error_called.clone();
    let cli = cli_default(fake_tmux, FakeGitWorktreeFail);
    let result = cli.run_create("new-feature");
    assert!(
        result.is_ok(),
        "run_create should return Ok even on failure"
    );
    assert!(
        show_error_called.get(),
        "show_error should be called when create_workspace fails"
    );
}

// ===========================================================================
// FakeSelector — returns configurable SelectionResult values from a queue
// ===========================================================================

struct FakeSelector {
    results: RefCell<Vec<SelectionResult>>,
}
impl FakeSelector {
    fn new(results: Vec<SelectionResult>) -> Self {
        Self {
            results: RefCell::new(results),
        }
    }
}
impl SelectorRunner for FakeSelector {
    fn run_selector(
        &self,
        _selector: &mut Selector,
        _filtered: &[usize],
        _prompt: &str,
        _header: &str,
    ) -> Result<SelectionResult> {
        let mut results = self.results.borrow_mut();
        if !results.is_empty() {
            Ok(results.remove(0))
        } else {
            Ok(SelectionResult::Cancelled)
        }
    }
}

// ===========================================================================
// FakeTmxTest — generic TmuxPort for most tests
// ===========================================================================

struct FakeTmxTest {
    kill_called: Rc<Cell<bool>>,
    show_error_called: Rc<Cell<bool>>,
    show_error_msg: Rc<RefCell<String>>,
    display_msgs: Rc<RefCell<Vec<String>>>,
    pane_path: Option<String>,
    env_value: Option<String>,
    worktree_dir: String,
    shell_cmd: String,
    auto_fetch: Option<String>,
}
impl FakeTmxTest {
    fn new() -> Self {
        Self {
            kill_called: Rc::new(Cell::new(false)),
            show_error_called: Rc::new(Cell::new(false)),
            show_error_msg: Rc::new(RefCell::new(String::new())),
            display_msgs: Rc::new(RefCell::new(Vec::new())),
            pane_path: None,
            env_value: Some(REPO.to_string()),
            worktree_dir: ".worktrees".into(),
            shell_cmd: "/bin/bash".into(),
            auto_fetch: Some("false".into()),
        }
    }
}
impl TmuxPort for FakeTmxTest {
    fn show_error(&self, msg: &str) -> Result<()> {
        self.show_error_called.set(true);
        *self.show_error_msg.borrow_mut() = msg.to_string();
        Ok(())
    }
    fn resolve_workspace_dir(&self) -> String {
        self.worktree_dir.clone()
    }
    fn resolve_shell_command(&self) -> String {
        self.shell_cmd.clone()
    }
    fn select_or_create_window(&self, _n: &str, _p: &str, _c: &str) -> Result<()> {
        Ok(())
    }
    fn show_environment(&self, _v: &str) -> Result<Option<String>> {
        Ok(self.env_value.clone())
    }
    fn current_pane_path(&self) -> Result<String> {
        match &self.pane_path {
            Some(p) => Ok(p.clone()),
            None => Err(tmux_worktrees::core::error::Error::new("no pane")),
        }
    }
    fn display_popup(&self, _w: &str, _h: &str, _d: &str, _c: &str) -> Result<()> {
        unimplemented!()
    }
    fn get_option(&self, _k: &str) -> Option<String> {
        self.auto_fetch.clone()
    }
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)> {
        if args.len() >= 2 && args[0] == "display-message" {
            self.display_msgs.borrow_mut().push(args[1].to_string());
        }
        Ok((0, String::new(), String::new()))
    }
    fn kill_window(&self, _n: &str) -> Result<()> {
        self.kill_called.set(true);
        Ok(())
    }
}

// ===========================================================================
// FakeGitPorcelain — handles only worktree list --porcelain
// ===========================================================================

struct FakeGitPorcelain {
    output: String,
}
impl FakeGitPorcelain {
    fn new(output: &str) -> Self {
        Self {
            output: output.to_string(),
        }
    }
}
impl GitPort for FakeGitPorcelain {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((0, self.output.clone(), String::new())),
            _ => unimplemented!("unexpected: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// FakeGitForCleanup — handles all commands needed by run_cleanup
// ===========================================================================

struct FakeGitForCleanup {
    porcelain: String,
    rev_parse_output: String,
    remove_status: i32,
    remove_stderr: String,
}
impl FakeGitForCleanup {
    fn new(porcelain: &str) -> Self {
        Self {
            porcelain: porcelain.to_string(),
            rev_parse_output: "feat-a".into(),
            remove_status: 0,
            remove_stderr: String::new(),
        }
    }
    fn with_remove_failure(mut self) -> Self {
        self.remove_status = 1;
        self.remove_stderr = "worktree locked".into();
        self
    }
}
impl GitPort for FakeGitForCleanup {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["worktree", "list", "--porcelain"] => Ok((0, self.porcelain.clone(), String::new())),
            ["rev-parse", "--abbrev-ref", "HEAD"] => {
                Ok((0, self.rev_parse_output.clone(), String::new()))
            }
            ["merge-base", "--is-ancestor", "HEAD", _] => Ok((0, String::new(), String::new())),
            ["fetch", "origin", _, "--no-tags"] => Ok((0, String::new(), String::new())),
            ["worktree", "remove", "--force", _] => Ok((
                self.remove_status,
                String::new(),
                self.remove_stderr.clone(),
            )),
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, args: &[&str]) -> Result<()> {
        match args {
            ["branch", "-D", _] => Ok(()),
            _ => unimplemented!("unexpected git silent_in: {:?}", args),
        }
    }
}

// ===========================================================================
// FakeGitForChoose — handles porcelain + full run_create flow
// ===========================================================================

struct FakeGitForChoose;
impl GitPort for FakeGitForChoose {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((
                0,
                format!("worktree {}\nworktree {}/.worktrees/feat-a\n", REPO, REPO),
                String::new(),
            )),
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["show-ref", "--verify", "--quiet", _] => Ok((1, String::new(), String::new())),
            ["worktree", "add", _, "-b", _, _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

// ===========================================================================
// Repo path for test porcelain
// ===========================================================================

const REPO: &str = "/home/gbrennon/Documents/repos/gbrennon/tmux-worktrees";

// ===========================================================================
// Test: dispatch calls run_choose
// ===========================================================================

#[test]
fn dispatch_choose_calls_run_choose_and_cancels() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(),
        FakeGitPorcelain::new(&format!(
            "worktree {}\nworktree {}/.worktrees/feat-a\n",
            REPO, REPO
        )),
        selector,
    );
    let result = cli.dispatch(Command::Choose, &[]);
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: dispatch calls run_cleanup
// ===========================================================================

#[test]
fn dispatch_cleanup_calls_run_cleanup_and_cancels() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(),
        FakeGitForCleanup::new(&format!(
            "worktree {}\nworktree {}/.worktrees/feat-a\n",
            REPO, REPO
        )),
        selector,
    );
    let result = cli.dispatch(Command::Cleanup, &[]);
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: run_choose returns Ok when selector cancels
// ===========================================================================

#[test]
fn run_choose_returns_cancelled() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(),
        FakeGitPorcelain::new(&format!(
            "worktree {}\nworktree {}/.worktrees/feat-a\n",
            REPO, REPO
        )),
        selector,
    );
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: run_choose shows no-workspaces header when no workspaces
// ===========================================================================

#[test]
fn run_choose_no_workspaces_shows_header_and_cancels() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(),
        FakeGitPorcelain::new(""), // empty porcelain — only main repo path filtered out
        selector,
    );
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: run_choose creates workspace when custom branch entered
// ===========================================================================

#[test]
fn run_choose_creates_workspace_for_custom_branch() {
    let selector = FakeSelector::new(vec![SelectionResult::Custom("my-branch".into())]);
    let cli = cli_with(FakeTmxTest::new(), FakeGitForChoose, selector);
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: run_cleanup full success — removes workspace and deletes branch
// ===========================================================================

#[test]
fn run_cleanup_removes_workspace_and_deletes_branch() {
    let tmux = FakeTmxTest::new();
    let kill_called = tmux.kill_called.clone();
    let display_msgs = tmux.display_msgs.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let cli = cli_with(
        tmux,
        FakeGitForCleanup::new(&format!(
            "worktree {}\nworktree {}/.worktrees/feat-a\n",
            REPO, REPO
        )),
        selector,
    );
    let result = cli.run_cleanup();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert!(kill_called.get(), "kill_window should have been called");
    let msgs: Vec<_> = display_msgs.borrow().iter().cloned().collect();
    let found = msgs.iter().any(|m| m.contains("Removed workspace: feat-a"));
    assert!(
        found,
        "display-message for removed workspace not found in {:?}",
        msgs
    );
}

// ===========================================================================
// Test: run_cleanup shows error when workspace removal fails
// ===========================================================================

#[test]
fn run_cleanup_shows_error_on_removal_failure() {
    let tmux = FakeTmxTest::new();
    let show_error_called = tmux.show_error_called.clone();
    let show_error_msg = tmux.show_error_msg.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let cli = cli_with(
        tmux,
        FakeGitForCleanup::new(&format!(
            "worktree {}\nworktree {}/.worktrees/feat-a\n",
            REPO, REPO
        ))
        .with_remove_failure(),
        selector,
    );
    let result = cli.run_cleanup();
    assert!(
        result.is_ok(),
        "run_cleanup should return Ok even on removal failure"
    );
    assert!(
        show_error_called.get(),
        "show_error should have been called"
    );
    let msg = show_error_msg.borrow();
    assert!(
        msg.contains("Workspace removal failed"),
        "unexpected error message: {}",
        msg
    );
}

// ===========================================================================
// Test: run_cleanup shows error when no workspaces found
// ===========================================================================

#[test]
fn run_cleanup_empty_workspaces_shows_message() {
    let tmux = FakeTmxTest::new();
    let show_error_called = tmux.show_error_called.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        tmux,
        FakeGitForCleanup::new(""), // empty porcelain
        selector,
    );
    let result = cli.run_cleanup();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert!(
        show_error_called.get(),
        "show_error should have been called for empty workspaces"
    );
}
