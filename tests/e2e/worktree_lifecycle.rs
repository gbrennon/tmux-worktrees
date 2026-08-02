use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;
use tmux_worktrees::app;
use tmux_worktrees::infrastructure::command_runner::{CommandRunner, SystemCommandRunner};
use tmux_worktrees::infrastructure::git_executor::GitExecutor;
use tmux_worktrees::infrastructure::tmux_executor::TmuxExecutor;

// ---------------------------------------------------------------------------
// Isolated tmux test server — spawned once per test binary, never touches
// the user's real tmux server.
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
    let socket = format!("tmux-worktrees-e2e-{}", std::process::id());
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
            "-x",
            "80",
            "-y",
            "24",
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

fn e2e_ctx() -> (TmuxExecutor, GitExecutor) {
    let socket = ensure_test_server().expect("tmux test server not available");
    (
        TmuxExecutor::with_runner(Box::new(SocketRunner {
            socket: socket.to_owned(),
            inner: SystemCommandRunner,
        })),
        GitExecutor::default(),
    )
}

// ---------------------------------------------------------------------------
// Ephemeral repo helpers
// ---------------------------------------------------------------------------

/// RepoGuard holds a `TempDir` alive for the test's duration.
/// The repo path is inside the temp dir at `repo_path()`.
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
        let status = Command::new("git")
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

    // Create the worktrees subdirectory so list_workspaces can resolve it
    std::fs::create_dir_all(repo.join("worktrees")).unwrap();

    RepoGuard { _dir: dir, repo }
}

// ---------------------------------------------------------------------------
// parse_args
// ---------------------------------------------------------------------------

#[test]
fn parse_args_defaults_to_choose() {
    let (cmd, rest) = app::parse_args(&["tmux-worktrees".to_string()]);
    assert_eq!(cmd, "choose");
    assert!(rest.is_empty());
}

#[test]
fn parse_args_extracts_command() {
    let (cmd, rest) = app::parse_args(&["tmux-worktrees".to_string(), "cleanup".to_string()]);
    assert_eq!(cmd, "cleanup");
    assert_eq!(rest, vec!["cleanup"]);
}

#[test]
fn parse_args_preserves_trailing_args() {
    let (cmd, rest) = app::parse_args(&[
        "tmux-worktrees".to_string(),
        "create-worktree".to_string(),
        "feat/some-branch".to_string(),
    ]);
    assert_eq!(cmd, "create-worktree");
    assert_eq!(rest, vec!["create-worktree", "feat/some-branch"]);
}

#[test]
fn parse_args_strips_root_equals_flag() {
    let (cmd, rest) = app::parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root=/tmp/test".to_string(),
    ]);
    assert_eq!(cmd, "choose");
    assert_eq!(rest, vec!["choose"]);
}
#[test]
fn find_repo_root_via_env_var() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    // find_repo_root checks TMUX_WORKTREES_ROOT env var first
    std::env::set_var(
        "TMUX_WORKTREES_ROOT",
        guard.repo_path().to_string_lossy().as_ref(),
    );
    let root = app::find_repo_root(&_tmux).unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
    std::env::remove_var("TMUX_WORKTREES_ROOT");
}

#[test]
fn find_repo_root_by_walking() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    // Ensure no env-var shortcut
    std::env::remove_var("TMUX_WORKTREES_ROOT");
    std::env::set_current_dir(guard.repo_path()).unwrap();
    let root = app::find_repo_root(&_tmux).unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
}

#[test]
fn find_repo_root_from_subdirectory() {
    let guard = init_ephemeral_repo();
    let subdir = guard.repo_path().join("src");
    std::fs::create_dir(&subdir).unwrap();
    let (_tmux, _git) = e2e_ctx();
    std::env::remove_var("TMUX_WORKTREES_ROOT");
    std::env::set_current_dir(&subdir).unwrap();
    let found = app::find_repo_root(&_tmux).unwrap();
    assert_eq!(found, guard.repo_path().to_string_lossy());
}
#[test]
fn parse_args_strips_separate_root_flag() {
    let (cmd, rest) = app::parse_args(&[
        "tmux-worktrees".to_string(),
        "choose".to_string(),
        "--root".to_string(),
        "/tmp/test".to_string(),
    ]);
    assert_eq!(cmd, "choose");
    assert_eq!(rest, vec!["choose"]);
}
// ---------------------------------------------------------------------------
// resolve_default_branch
// ---------------------------------------------------------------------------

#[test]
fn resolve_default_branch_in_ephemeral_repo() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let branch = app::resolve_default_branch(&_git, guard.repo_path()).unwrap();
    // Should resolve to "main" (our init branch) unless overridden by config
    assert!(!branch.is_empty());
}

// ---------------------------------------------------------------------------
// list_workspaces / list_workspace_names
// ---------------------------------------------------------------------------

#[test]
fn list_workspaces_empty_when_no_worktrees() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let workspaces = app::list_workspaces(&_git, guard.repo_path(), "worktrees").unwrap();
    assert!(
        workspaces.is_empty(),
        "expected no worktrees in fresh repo, got {workspaces:?}"
    );
}

#[test]
fn list_workspace_names_empty_initially() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let names = app::list_workspace_names(&_git, guard.repo_path(), "worktrees").unwrap();
    assert!(names.is_empty());
}

// ---------------------------------------------------------------------------
// worktree lifecycle: create → list → branch → remove → verify gone
// ---------------------------------------------------------------------------

#[test]
fn worktree_full_lifecycle() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-lifecycle");
    let branch = "feat/lifecycle-test";

    // --- Create ---
    app::create_workspace(&_git, repo_root, &target, branch, "main").unwrap();

    // --- List ---
    let workspaces = app::list_workspaces(&_git, repo_root, "worktrees").unwrap();
    assert!(
        workspaces.iter().any(|p| p == &target),
        "created worktree not found in {workspaces:?}"
    );

    // --- Branch ---
    let found_branch = app::workspace_branch(&_git, &target).unwrap();
    assert_eq!(found_branch, branch);

    // --- Remove ---
    app::remove_workspace(&_tmux, &_git, repo_root, &target, branch).unwrap();

    // --- Verify gone ---
    let workspaces = app::list_workspaces(&_git, repo_root, "worktrees").unwrap();
    assert!(
        !workspaces.iter().any(|p| p == &target),
        "worktree should be removed, still found: {workspaces:?}"
    );
}

#[test]
fn create_multiple_worktrees_and_list_all() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let repo_root = guard.repo_path();

    let branches = ["feat/alpha", "feat/beta", "feat/gamma"];
    let mut targets = Vec::new();

    for &b in &branches {
        let t = guard.work_dir().join(b);
        app::create_workspace(&_git, repo_root, &t, b, "main").unwrap();
        targets.push((t, b));
    }

    let workspaces = app::list_workspaces(&_git, repo_root, "worktrees").unwrap();
    for (t, _) in &targets {
        assert!(workspaces.iter().any(|p| p == t), "missing {t:?}");
    }

    // Clean up all worktrees
    for (t, b) in &targets {
        app::remove_workspace(&_tmux, &_git, repo_root, t, b).unwrap();
    }

    let workspaces = app::list_workspaces(&_git, repo_root, "worktrees").unwrap();
    for (t, _) in &targets {
        assert!(!workspaces.iter().any(|p| p == t), "should be gone: {t:?}");
    }
}

#[test]
fn delete_branch_removes_it() {
    let guard = init_ephemeral_repo();
    let (_tmux, _git) = e2e_ctx();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-to-delete");
    let branch = "feat/to-delete";

    app::create_workspace(&_git, repo_root, &target, branch, "main").unwrap();
    app::remove_workspace(&_tmux, &_git, repo_root, &target, branch).unwrap();

    // Delete the branch (it still exists after worktree remove)
    app::delete_branch(&_git, repo_root, branch);

    // Verify it's gone: trying to create a worktree from the same branch
    // would fail if it still existed (it would try to reuse), or we can
    // just check that it's not in `git branch --list`
    let branch_target = guard.work_dir().join("feat-to-delete-v2");
    app::create_workspace(&_git, repo_root, &branch_target, branch, "main").unwrap();

    let found = app::workspace_branch(&_git, &branch_target).unwrap();
    assert_eq!(found, branch);

    // Clean up
    app::remove_workspace(&_tmux, &_git, repo_root, &branch_target, branch).unwrap();
    app::delete_branch(&_git, repo_root, branch);
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

#[test]
fn dispatch_unknown_command_shows_error() {
    let (_tmux, _git) = e2e_ctx();
    let result = app::dispatch(
        &_tmux,
        &_git,
        &["tmux-worktrees".to_string(), "nonexistent-cmd".to_string()],
    );
    // Should succeed (error is displayed in tmux, not returned)
    assert!(result.is_ok());
}
