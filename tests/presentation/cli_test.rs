use std::path::Path;

use super::common::fakes::git::*;
use super::common::fakes::loading::*;
use super::common::fakes::selector::*;
use super::common::fakes::tmux::*;
use tmux_worktrees::core::ports::{GitPort, TmuxPort};
use tmux_worktrees::core::selector::SelectionResult;
use tmux_worktrees::presentation::cli::Cli;
use tmux_worktrees::presentation::command::Command;
use tmux_worktrees::presentation::loading_port::LoadingRunner;
use tmux_worktrees::presentation::selector_port::SelectorRunner;

fn cli_with(
    tmux: impl TmuxPort + 'static,
    git: impl GitPort + 'static,
    selector: impl SelectorRunner + 'static,
) -> Cli {
    cli_with_loading(tmux, git, selector, FakeLoading::new())
}

fn cli_with_loading(
    tmux: impl TmuxPort + 'static,
    git: impl GitPort + 'static,
    selector: impl SelectorRunner + 'static,
    loading: impl LoadingRunner + 'static,
) -> Cli {
    Cli::new(
        Box::new(tmux),
        Box::new(git),
        Box::new(selector),
        Box::new(loading),
    )
}

fn cli_default(tmux: impl TmuxPort + 'static, git: impl GitPort + 'static) -> Cli {
    cli_with(tmux, git, StubSelector)
}

fn linked_worktree_root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(
        root.path().join(".git"),
        "gitdir: /some/where/.git/worktrees/test\n",
    )
    .unwrap();
    root
}

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

#[test]
fn dispatch_create_worktree_empty_branch_shows_error() {
    let cli = cli_default(FakeTmxShowError, StubGit);
    let result = cli.dispatch(Command::CreateWorktree, &[]);
    assert!(result.is_ok());
}

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
    let root = linked_worktree_root();
    unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root.path()) };
    let result = cli.spawn_in_popup(&["tmux-worktrees".to_string(), "choose".to_string()]);
    assert!(result.is_ok());
    assert!(!guard.show_error_called.get());
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
}

#[test]
fn spawn_in_popup_calls_display_popup_when_root_found() {
    let root = linked_worktree_root();
    unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root.path()) };
    let tmux = FakeTmxPopupOk::new();
    let guard = tmux.clone();
    let cli = cli_default(tmux, StubGit);
    let result = cli.spawn_in_popup(&["tmux-worktrees".to_string(), "choose".to_string()]);
    unsafe {
        std::env::remove_var("TMUX_WORKTREES_ROOT");
    }
    assert!(result.is_ok());
    assert!(guard.display_called.get());
}

#[test]
fn spawn_in_popup_accepts_a_main_repository() {
    let temporary_repository = tempfile::tempdir().unwrap();
    std::fs::create_dir(temporary_repository.path().join(".git")).unwrap();
    unsafe {
        std::env::set_var("TMUX_WORKTREES_ROOT", temporary_repository.path());
    }

    let tmux = FakeTmxPopupOk::new();
    let guard = tmux.clone();
    let cli = cli_default(tmux, StubGit);
    let result = cli.spawn_in_popup(&["tmux-worktrees".to_string(), "choose".to_string()]);

    unsafe {
        std::env::remove_var("TMUX_WORKTREES_ROOT");
    }
    assert!(result.is_ok());
    assert!(guard.display_called.get());
}

#[test]
fn run_create_shows_error_on_empty_branch() {
    let cli = cli_default(FakeTmxShowError, StubGit);
    let result = cli.run_create("");
    assert!(result.is_ok());
}

#[test]
fn create_worktree_uses_origin_prefix_when_remote_exists() {
    let cli = cli_default(StubTmx, FakeGitWorktreeAdd);
    let result = cli.create_worktree(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_ok());
}

#[test]
fn create_worktree_uses_local_branch_when_no_remote() {
    let cli = cli_default(StubTmx, FakeGitWorktreeAddLocal);
    let result = cli.create_worktree(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_ok());
}

#[test]
fn create_worktree_propagates_git_errors() {
    let cli = cli_default(StubTmx, FakeGitWorktreeFail);
    let result = cli.create_worktree(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_err());
}

#[test]
fn remove_worktree_skips_kill_window_when_branch_empty() {
    let cli = cli_default(StubTmx, FakeGitWorktreeRemove);
    let result = cli.remove_worktree(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "");
    assert!(result.is_ok());
}

#[test]
fn remove_worktree_with_branch_calls_kill_window() {
    let cli = cli_default(FakeTmxWithWindow, FakeGitWorktreeRemove);
    let result = cli.remove_worktree(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "feat/x");
    assert!(result.is_ok());
}

#[test]
fn remove_worktree_propagates_git_error() {
    let cli = cli_default(FakeTmxNoWindow, FakeGitWorktreeRemoveFail);
    let result = cli.remove_worktree(Path::new("/tmp/repo"), Path::new("/tmp/ws"), "feat/x");
    assert!(result.is_err());
}

#[test]
fn worktree_branch_returns_question_mark_on_failure() {
    let cli = cli_default(StubTmx, FakeGitRevParseFail);
    let result = cli.worktree_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "?");
}

#[test]
fn worktree_branch_returns_branch_name_on_success() {
    let cli = cli_default(StubTmx, FakeGitRevParseOk);
    let result = cli.worktree_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "feat/x\n");
}

#[test]
fn worktree_branch_returns_detached_head() {
    let cli = cli_default(StubTmx, FakeGitRevParseDetached);
    let result = cli.worktree_branch(Path::new("/tmp/ws"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "HEAD\n");
}

#[test]
fn list_worktrees_returns_empty_on_nonzero_status() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListFail);
    let result = cli.list_worktrees(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn list_worktrees_returns_worktree_paths() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListOk);
    let result = cli.list_worktrees(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    let paths = result.unwrap();
    assert_eq!(paths.len(), 2);
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("feat-a")));
    assert!(paths.iter().any(|p| p.to_string_lossy().contains("fix-b")));
}

#[test]
fn list_worktrees_includes_worktrees_outside_configured_directory() {
    let cli = cli_default(
        StubTmx,
        FakeGitPorcelain::new(
            "worktree /tmp/repo\nworktree /tmp/repo/worktrees/feat-a\nworktree /tmp/external-worktree\n",
        ),
    );
    let paths = cli
        .list_worktrees(Path::new("/tmp/repo"), "worktrees")
        .unwrap();

    assert_eq!(
        paths,
        vec![
            Path::new("/tmp/repo/worktrees/feat-a"),
            Path::new("/tmp/external-worktree"),
        ]
    );
}

#[test]
fn list_worktree_names_extracts_sorted_names() {
    let cli = cli_default(StubTmx, FakeGitWorktreeListOk);
    let result = cli.list_worktree_names(Path::new("/tmp/repo"), "worktrees");
    assert!(result.is_ok());
    let names = result.unwrap();
    assert_eq!(names, vec!["feat-a", "fix-b"]);
}

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

#[test]
fn delete_branch_does_not_panic() {
    let cli = cli_default(StubTmx, FakeGitBranchDelete);
    cli.delete_branch(Path::new("/tmp/repo"), "feat/x").unwrap();
}

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

#[test]
fn run_create_full_success_path() {
    let cli = cli_default(FakeTmxCreateSuccess, FakeGitCreate);
    let result = cli.dispatch(Command::CreateWorktree, &["test-branch".to_string()]);
    assert!(result.is_ok(), "expected Ok, got {result:?}");
}

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

#[test]
fn run_create_shows_error_when_worktree_creation_fails() {
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
        "show_error should be called when create_worktree fails"
    );
}

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

#[test]
fn run_choose_no_worktrees_shows_header_and_cancels() {
    let selector = FakeSelector::new(vec![SelectionResult::Cancelled]);
    let cli = cli_with(
        FakeTmxTest::new(REPO),
        FakeGitPorcelain::new(""), // empty porcelain — only main repo path filtered out
        selector,
    );
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

#[test]
fn run_choose_creates_worktree_for_custom_branch() {
    let selector = FakeSelector::new(vec![SelectionResult::Custom("my-branch".into())]);
    let cli = cli_with(
        FakeTmxTest::new(REPO),
        FakeGitForChoose::new(REPO),
        selector,
    );
    let result = cli.run_choose();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
}

#[test]
fn run_choose_opens_existing_worktree_after_branch_rename() {
    let temporary_repository = tempfile::tempdir().unwrap();
    let repository_path = temporary_repository.path();
    let existing_worktree = temporary_repository.path().join("external-worktree");
    std::fs::create_dir(repository_path.join(".git")).unwrap();

    let previous_directory = std::env::current_dir().unwrap();
    let previous_root = std::env::var_os("TMUX_WORKTREES_ROOT");
    unsafe {
        std::env::set_var("TMUX_WORKTREES_ROOT", repository_path);
    }

    let tmux = FakeTmxTest::new(&repository_path.to_string_lossy());
    let selected_window = tmux.selected_window.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let git = FakeGitForChoose::existing_worktree(
        &repository_path.to_string_lossy(),
        &existing_worktree.to_string_lossy(),
        "renamed/name",
    );
    let cli = cli_with(tmux, git, selector);

    let names = cli
        .list_worktree_names(repository_path, ".worktrees")
        .unwrap();
    assert_eq!(names, vec!["external-worktree"]);
    let result = cli.run_choose();

    match previous_root {
        Some(root) => unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root) },
        None => unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") },
    }
    std::env::set_current_dir(previous_directory).unwrap();
    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(
        selected_window.borrow().as_ref(),
        Some(&(
            "renamed/name".to_string(),
            existing_worktree.to_string_lossy().into_owned(),
            "/bin/bash".to_string(),
        ))
    );
}

#[test]
fn run_cleanup_removes_worktree_and_deletes_branch() {
    let tmux = FakeTmxTest::new(REPO);
    let kill_called = tmux.kill_called.clone();
    let display_msgs = tmux.display_msgs.clone();
    let git = FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n");
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0]),
        SelectionResult::Selected(0),
    ]);
    let cli = cli_with(tmux, git, selector);
    let result = cli.run_cleanup();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert!(kill_called.get(), "kill_window should have been called");
    assert_eq!(remove_calls.borrow().len(), 1);
    assert_eq!(deleted_branches.borrow().as_slice(), ["feat-a"]);
    let msgs: Vec<_> = display_msgs.borrow().iter().cloned().collect();
    let found = msgs.iter().any(|m| m.contains("Removed worktree: feat-a"));
    assert!(
        found,
        "display-message for removed worktree not found in {:?}",
        msgs
    );
}

#[test]
fn run_cleanup_shows_error_on_removal_failure() {
    let tmux = FakeTmxTest::new(REPO);
    let show_error_called = tmux.show_error_called.clone();
    let show_error_msg = tmux.show_error_msg.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0]),
        SelectionResult::Selected(0),
    ]);
    let git = FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n")
        .with_remove_failure();
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let cli = cli_with(tmux, git, selector);
    let result = cli.run_cleanup();
    assert!(
        result.is_ok(),
        "run_cleanup should return Ok even on removal failure"
    );
    assert!(
        show_error_called.get(),
        "show_error should have been called"
    );
    assert_eq!(remove_calls.borrow().len(), 1);
    assert!(deleted_branches.borrow().is_empty());
    let msg = show_error_msg.borrow();
    assert!(
        msg.contains("Worktree removal failed"),
        "unexpected error message: {}",
        msg
    );
}

#[test]
fn run_cleanup_empty_selection_shows_error_without_confirmation() {
    let tmux = FakeTmxTest::new(REPO);
    let show_error_called = tmux.show_error_called.clone();
    let show_error_msg = tmux.show_error_msg.clone();
    let git = FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n");
    let remove_calls = git.remove_calls.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(Vec::new()),
        SelectionResult::Selected(0),
    ]);
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert!(show_error_called.get());
    assert!(
        show_error_msg
            .borrow()
            .contains("No worktrees were selected")
    );
    assert!(remove_calls.borrow().is_empty());
}
#[test]
fn run_cleanup_removes_selected_worktrees_in_display_order() {
    let tmux = FakeTmxTest::new(REPO);
    let display_msgs = tmux.display_msgs.clone();
    let kill_windows = tmux.kill_windows.clone();
    let porcelain = "worktree __ROOT__\n\
        worktree __ROOT__/.worktrees/feat-a\n\
        worktree __ROOT__/.worktrees/feat-b\n\
        worktree __ROOT__/.worktrees/feat-c\n";
    let git = FakeGitForCleanup::new(porcelain).with_branches(&["feat-a", "feat-b", "feat-c"]);
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0, 2]),
        SelectionResult::Selected(0),
    ]);
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert_eq!(kill_windows.borrow().as_slice(), ["feat-a", "feat-c"]);
    let calls = remove_calls.borrow();
    assert_eq!(calls.len(), 2);
    assert!(calls[0].ends_with("feat-a"));
    assert!(calls[1].ends_with("feat-c"));
    assert_eq!(deleted_branches.borrow().as_slice(), ["feat-a", "feat-c"]);
    let messages = display_msgs.borrow();
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("Removed worktree: feat-a"))
    );
    assert!(
        messages
            .iter()
            .any(|msg| msg.contains("Removed worktree: feat-c"))
    );
    assert!(
        !messages
            .iter()
            .any(|msg| msg.contains("Removed worktree: feat-b"))
    );
}

#[test]
fn run_cleanup_stops_after_middle_removal_failure() {
    let tmux = FakeTmxTest::new(REPO);
    let show_error_msg = tmux.show_error_msg.clone();
    let display_msgs = tmux.display_msgs.clone();
    let porcelain = "worktree __ROOT__\n\
        worktree __ROOT__/.worktrees/feat-a\n\
        worktree __ROOT__/.worktrees/feat-b\n\
        worktree __ROOT__/.worktrees/feat-c\n";
    let git = FakeGitForCleanup::new(porcelain)
        .with_branches(&["feat-a", "feat-b", "feat-c"])
        .with_remove_outcomes(&[(0, ""), (1, "second worktree locked"), (0, "")]);
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0, 1, 2]),
        SelectionResult::Selected(0),
    ]);
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert_eq!(remove_calls.borrow().len(), 2);
    assert!(remove_calls.borrow()[0].ends_with("feat-a"));
    assert!(remove_calls.borrow()[1].ends_with("feat-b"));
    assert_eq!(deleted_branches.borrow().as_slice(), ["feat-a"]);
    let messages = display_msgs.borrow();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("Removed worktree: feat-a"))
    );
    assert!(!messages.iter().any(|message| message.contains("feat-b")));
    assert!(!messages.iter().any(|message| message.contains("feat-c")));
    assert!(show_error_msg.borrow().contains("Worktree removal failed"));
}

#[test]
fn run_cleanup_stops_when_branch_deletion_fails() {
    let tmux = FakeTmxTest::new(REPO);
    let show_error_msg = tmux.show_error_msg.clone();
    let display_msgs = tmux.display_msgs.clone();
    let porcelain = "worktree __ROOT__\n\
        worktree __ROOT__/.worktrees/feat-a\n\
        worktree __ROOT__/.worktrees/feat-b\n\
        worktree __ROOT__/.worktrees/feat-c\n";
    let git = FakeGitForCleanup::new(porcelain)
        .with_branches(&["feat-a", "feat-b", "feat-c"])
        .with_branch_delete_outcomes(&[(1, "branch locked"), (0, ""), (0, "")]);
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0, 1, 2]),
        SelectionResult::Selected(0),
    ]);
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert_eq!(remove_calls.borrow().len(), 1);
    assert!(deleted_branches.borrow().is_empty());
    assert!(display_msgs.borrow().is_empty());
    assert!(show_error_msg.borrow().contains("Branch deletion failed"));
    assert!(show_error_msg.borrow().contains("branch locked"));
}
#[test]
fn run_cleanup_confirmation_escape_does_not_remove_worktrees() {
    let tmux = FakeTmxTest::new(REPO);
    let git = FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n");
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0]),
        SelectionResult::Cancelled,
    ]);
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    assert!(remove_calls.borrow().is_empty());
    assert!(deleted_branches.borrow().is_empty());
}
#[test]
fn run_cleanup_confirmation_cancel_does_not_remove_worktrees() {
    let tmux = FakeTmxTest::new(REPO);
    let git = FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n");
    let remove_calls = git.remove_calls.clone();
    let deleted_branches = git.deleted_branches.clone();
    let selector = FakeSelector::new(vec![
        SelectionResult::SelectedMany(vec![0]),
        SelectionResult::Selected(1),
    ]);
    let selector_calls = selector.calls.clone();
    let cli = cli_with(tmux, git, selector);

    let result = cli.run_cleanup();

    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let calls = selector_calls.borrow();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1], vec!["Yes".to_string(), "No".to_string()]);
    assert!(remove_calls.borrow().is_empty());
    assert!(deleted_branches.borrow().is_empty());
}
#[test]
fn run_cleanup_empty_worktrees_shows_message() {
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
        "show_error should have been called for empty worktrees"
    );
}

#[test]
fn run_cleanup_shows_loading_during_fetch() {
    let loading = FakeLoading::new();
    let loading_labels = loading.labels.clone();
    let mut tmux = FakeTmxTest::new(REPO);
    tmux.auto_fetch = None;
    let cli = cli_with_loading(
        tmux,
        FakeGitForCleanup::new("worktree __ROOT__\nworktree __ROOT__/.worktrees/feat-a\n"),
        FakeSelector::new(vec![SelectionResult::Cancelled]),
        loading,
    );
    let result = cli.run_cleanup();
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let labels: Vec<_> = loading_labels.borrow().iter().cloned().collect();
    assert!(
        labels.iter().any(|l| l.contains("Fetching")),
        "expected a fetch loading label, got {labels:?}"
    );
}

#[test]
fn create_worktree_shows_loading_during_fetch() {
    let loading = FakeLoading::new();
    let loading_labels = loading.labels.clone();
    let cli = cli_with_loading(StubTmx, FakeGitWorktreeAdd, StubSelector, loading);
    let result = cli.create_worktree(
        Path::new("/tmp/repo"),
        Path::new("/tmp/test-target"),
        "feat/x",
        "main",
    );
    assert!(result.is_ok(), "expected Ok, got {:?}", result);
    let labels: Vec<_> = loading_labels.borrow().iter().cloned().collect();
    assert!(
        labels.iter().any(|l| l.contains("Fetching")),
        "expected a fetch loading label, got {labels:?}"
    );
}
