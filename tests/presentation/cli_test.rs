// Integration tests for the Cli presentation layer.
// Each fake covers exactly one scenario — no god objects.

#[path = "../common/mod.rs"]
mod common;

use std::path::Path;

use common::fakes::git::*;
use common::fakes::selector::*;
use common::fakes::tmux::*;
use tmux_worktrees::core::ports::{GitPort, TmuxPort};
use tmux_worktrees::core::selector::SelectionResult;
use tmux_worktrees::presentation::cli::Cli;
use tmux_worktrees::presentation::command::Command;
use tmux_worktrees::presentation::selector_port::SelectorRunner;

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

/// Repo path for tests that rely on worktree list porcelain.
const REPO: &str = "/home/gbrennon/Documents/repos/gbrennon/tmux-worktrees";

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
#[test]
fn dispatch_choose_calls_run_choose_and_cancels() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(REPO),
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
        FakeTmxTest::new(REPO),
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
        FakeTmxTest::new(REPO),
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
        FakeTmxTest::new(REPO),
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
    let cli = cli_with(
        FakeTmxTest::new(REPO),
        FakeGitForChoose::new(REPO),
        selector,
    );
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

// ===========================================================================
// Test: run_cleanup full success — removes workspace and deletes branch
// ===========================================================================

#[test]
fn run_cleanup_removes_workspace_and_deletes_branch() {
    let tmux = FakeTmxTest::new(REPO);
    let kill_called = tmux.kill_called.clone();
    let display_msgs = tmux.display_msgs.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let cli = cli_with(
        tmux,
        FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n"),
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
    let tmux = FakeTmxTest::new(REPO);
    let show_error_called = tmux.show_error_called.clone();
    let show_error_msg = tmux.show_error_msg.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let cli = cli_with(
        tmux,
        FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n")
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
    let tmux = FakeTmxTest::new(REPO);
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
