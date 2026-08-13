use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::sync::Mutex;

use tempfile::TempDir;
use tmux_worktrees::presentation::cli::Cli;
use tmux_worktrees::presentation::command::Command as AppCommand;

use super::common::fakes::git::*;
use super::common::fakes::loading::*;
use super::common::fakes::selector::*;
use super::common::fakes::tmux::*;
use tmux_worktrees::core::ports::{GitPort, TmuxPort};
use tmux_worktrees::presentation::loading_port::LoadingRunner;
use tmux_worktrees::presentation::selector_port::SelectorRunner;

static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn cli_with(
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

fn cli_stub() -> Cli {
    cli_with(StubTmx, StubGit, StubSelector, FakeLoading::new())
}

/// RepoGuard holds a `TempDir` alive for the test's duration.
struct RepoGuard {
    _dir: TempDir,
    repo: PathBuf,
}

impl RepoGuard {
    fn repo_path(&self) -> &Path {
        &self.repo
    }

    fn work_dir(&self) -> PathBuf {
        self.repo.join("worktrees")
    }
}

/// Creates a temporary git repo with at least one commit (required for
/// `git worktree add`).
fn init_ephemeral_repo() -> RepoGuard {
    let dir = TempDir::new().expect("failed to create temp dir");
    let repo = dir.path().join("repo");
    std::fs::create_dir(&repo).unwrap();

    let run_git = |args: &[&str]| {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(&repo)
            .status()
            .expect("git command failed");
        assert!(status.success(), "git {:?} failed", args);
    };

    run_git(&["init"]);
    run_git(&[
        "-c",
        "user.name=e2e-test",
        "-c",
        "user.email=e2e@test",
        "commit",
        "--allow-empty",
        "-m",
        "init",
    ]);
    run_git(&["branch", "-M", "main"]);

    std::fs::create_dir_all(repo.join("worktrees")).unwrap();

    RepoGuard { _dir: dir, repo }
}

#[test]
fn parse_args_defaults_to_choose() {
    let cli = cli_stub();
    let (cmd, rest) = cli.parse_args(&["tmux-worktrees".to_string()]);
    assert_eq!(cmd, AppCommand::Choose);
    assert!(rest.is_empty());
}

#[test]
fn parse_args_extracts_command() {
    let cli = cli_stub();
    let (cmd, rest) = cli.parse_args(&["tmux-worktrees".to_string(), "cleanup".to_string()]);
    assert_eq!(cmd, AppCommand::Cleanup);
    assert!(rest.is_empty());
}

#[test]
fn parse_args_preserves_trailing_args() {
    let cli = cli_stub();
    let (cmd, rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "create-worktree".to_string(),
        "feat/some-branch".to_string(),
    ]);
    assert_eq!(cmd, AppCommand::CreateWorktree);
    assert_eq!(rest, vec!["feat/some-branch"]);
}

#[test]
fn parse_args_strips_root_equals_flag() {
    let cli = cli_stub();
    let (cmd, _rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root=/tmp/test".to_string(),
    ]);
    assert_eq!(cmd, AppCommand::Choose);
    assert!(_rest.is_empty());
}

#[test]
fn parse_args_strips_separate_root_flag() {
    let _guard = ENV_LOCK.lock().unwrap();
    let cli = cli_stub();
    let (cmd, rest) = cli.parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root".to_string(),
        "/tmp/test".to_string(),
    ]);
    assert_eq!(cmd, AppCommand::Choose);
    assert!(rest.is_empty());
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
}

#[test]
fn unknown_command_defaults_to_choose() {
    let cli = cli_stub();
    let (cmd, rest) =
        cli.parse_args(&["tmux-worktrees".to_string(), "nonexistent-cmd".to_string()]);
    assert_eq!(cmd, AppCommand::Choose);
    assert!(rest.is_empty());
}

#[test]
fn find_repo_root_via_env_var() {
    let _guard = ENV_LOCK.lock().unwrap();
    let guard = init_ephemeral_repo();
    let cli = cli_stub();
    unsafe {
        std::env::set_var(
            "TMUX_WORKTREES_ROOT",
            guard.repo_path().to_string_lossy().as_ref(),
        )
    };
    let root = cli.find_repo_root().unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
}

#[test]
fn find_repo_root_by_walking() {
    let _guard = ENV_LOCK.lock().unwrap();
    let guard = init_ephemeral_repo();
    let cli = cli_stub();
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
    let root = cli.find_repo_root_from(guard.repo_path()).unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
}

#[test]
fn find_repo_root_from_subdirectory() {
    let _guard = ENV_LOCK.lock().unwrap();
    let guard = init_ephemeral_repo();
    let subdir = guard.repo_path().join("src");
    std::fs::create_dir(&subdir).unwrap();
    let cli = cli_stub();
    unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
    let found = cli.find_repo_root_from(&subdir).unwrap();
    assert_eq!(found, guard.repo_path().to_string_lossy());
}

#[test]
fn resolve_default_branch_in_ephemeral_repo() {
    let guard = init_ephemeral_repo();
    let cli = cli_with(
        StubTmx,
        FakeGitDefaultBranch,
        StubSelector,
        FakeLoading::new(),
    );
    let branch = cli.resolve_default_branch(guard.repo_path()).unwrap();
    assert!(!branch.is_empty());
}

#[test]
fn list_workspaces_empty_when_no_worktrees() {
    let guard = init_ephemeral_repo();
    let cli = cli_with(
        StubTmx,
        FakeGitWorktreeListOk,
        StubSelector,
        FakeLoading::new(),
    );
    let workspaces = cli.list_workspaces(guard.repo_path(), "worktrees").unwrap();
    assert!(
        workspaces.is_empty(),
        "expected no worktrees in fresh repo, got {workspaces:?}"
    );
}

#[test]
fn list_workspace_names_empty_initially() {
    let guard = init_ephemeral_repo();
    let cli = cli_with(
        StubTmx,
        FakeGitWorktreeListOk,
        StubSelector,
        FakeLoading::new(),
    );
    let names = cli
        .list_workspace_names(guard.repo_path(), "worktrees")
        .unwrap();
    assert!(names.is_empty());
}

#[test]
fn worktree_full_lifecycle() {
    let guard = init_ephemeral_repo();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-lifecycle");
    let branch = "feat/lifecycle-test";

    let git = FakeGitLifecycle::new();
    let tmux = FakeTmxTest::new(&repo_root.to_string_lossy());
    let kill_called = tmux.kill_called.clone();

    let cli = cli_with(tmux, git, StubSelector, FakeLoading::new());

    cli.create_workspace(repo_root, &target, branch, "main")
        .unwrap();

    let workspaces = cli.list_workspaces(repo_root, "worktrees").unwrap();
    assert!(workspaces.is_empty());

    let found_branch = cli.workspace_branch(&target).unwrap();
    assert_eq!(found_branch, branch);

    cli.remove_workspace(repo_root, &target, branch).unwrap();

    assert!(kill_called.get());
}

#[test]
fn create_multiple_worktrees_and_list_all() {
    let guard = init_ephemeral_repo();
    let repo_root = guard.repo_path();
    let git = FakeGitLifecycle::new();
    let tmux = FakeTmxTest::new(&repo_root.to_string_lossy());
    let kill_called = tmux.kill_called.clone();

    let cli = cli_with(tmux, git, StubSelector, FakeLoading::new());

    let branches = ["feat/alpha", "feat/beta", "feat/gamma"];
    let mut targets = Vec::new();

    for &b in &branches {
        let t = guard.work_dir().join(b);
        cli.create_workspace(repo_root, &t, b, "main").unwrap();
        targets.push((t, b));
    }

    assert_eq!(targets.len(), 3);

    for (t, b) in &targets {
        cli.remove_workspace(repo_root, t, b).unwrap();
    }

    assert!(kill_called.get());
}

#[test]
fn delete_branch_removes_it() {
    let guard = init_ephemeral_repo();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-to-delete");
    let branch = "feat/to-delete";

    let git = FakeGitLifecycle::new();
    let tmux = FakeTmxTest::new(&repo_root.to_string_lossy());

    let cli = cli_with(tmux, git, StubSelector, FakeLoading::new());

    cli.create_workspace(repo_root, &target, branch, "main")
        .unwrap();
    cli.remove_workspace(repo_root, &target, branch).unwrap();

    cli.delete_branch(repo_root, branch);

    let branch_target = guard.work_dir().join("feat-to-delete-v2");
    cli.create_workspace(repo_root, &branch_target, branch, "main")
        .unwrap();

    let found = cli.workspace_branch(&branch_target).unwrap();
    assert_eq!(found, branch);

    cli.remove_workspace(repo_root, &branch_target, branch)
        .unwrap();
    cli.delete_branch(repo_root, branch);
}
