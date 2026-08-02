use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::LazyLock;

use tempfile::TempDir;
use tmux_worktrees::infrastructure::command_runner::{CommandRunner, SystemCommandRunner};
use tmux_worktrees::infrastructure::git_executor::GitExecutor;
use tmux_worktrees::infrastructure::tmux_executor::TmuxExecutor;
use tmux_worktrees::pipeline::{self, Ctx};

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

fn e2e_ctx() -> Ctx {
    let socket = ensure_test_server().expect("tmux test server not available");
    Ctx {
        tmux: TmuxExecutor::with_runner(Box::new(SocketRunner {
            socket: socket.to_owned(),
            inner: SystemCommandRunner,
        })),
        git: GitExecutor::default(),
    }
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
    let (cmd, rest) = pipeline::parse_args(&["tmux-worktrees".to_string()]);
    assert_eq!(cmd, "choose");
    assert!(rest.is_empty());
}

#[test]
fn parse_args_extracts_command() {
    let (cmd, rest) = pipeline::parse_args(&["tmux-worktrees".to_string(), "cleanup".to_string()]);
    assert_eq!(cmd, "cleanup");
    assert_eq!(rest, vec!["cleanup"]);
}

#[test]
fn parse_args_preserves_trailing_args() {
    let (cmd, rest) = pipeline::parse_args(&[
        "tmux-worktrees".to_string(),
        "create-worktree".to_string(),
        "feat/some-branch".to_string(),
    ]);
    assert_eq!(cmd, "create-worktree");
    assert_eq!(rest, vec!["create-worktree", "feat/some-branch"]);
}

#[test]
fn parse_args_strips_root_equals_flag() {
    let (cmd, rest) = pipeline::parse_args(&[
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
    let ctx = e2e_ctx();
    // find_repo_root checks TMUX_WORKTREES_ROOT env var first
    std::env::set_var(
        "TMUX_WORKTREES_ROOT",
        guard.repo_path().to_string_lossy().as_ref(),
    );
    let root = pipeline::find_repo_root(&ctx).unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
    std::env::remove_var("TMUX_WORKTREES_ROOT");
}

#[test]
fn find_repo_root_by_walking() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    // Ensure no env-var shortcut
    std::env::remove_var("TMUX_WORKTREES_ROOT");
    std::env::set_current_dir(guard.repo_path()).unwrap();
    let root = pipeline::find_repo_root(&ctx).unwrap();
    assert_eq!(root, guard.repo_path().to_string_lossy());
}

#[test]
fn find_repo_root_from_subdirectory() {
    let guard = init_ephemeral_repo();
    let subdir = guard.repo_path().join("src");
    std::fs::create_dir(&subdir).unwrap();
    let ctx = e2e_ctx();
    std::env::remove_var("TMUX_WORKTREES_ROOT");
    std::env::set_current_dir(&subdir).unwrap();
    let found = pipeline::find_repo_root(&ctx).unwrap();
    assert_eq!(found, guard.repo_path().to_string_lossy());
}
#[test]
fn parse_args_strips_separate_root_flag() {
    let (cmd, rest) = pipeline::parse_args(&[
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
    let ctx = e2e_ctx();
    let branch = pipeline::resolve_default_branch(&ctx, guard.repo_path()).unwrap();
    // Should resolve to "main" (our init branch) unless overridden by config
    assert!(!branch.is_empty());
}

// ---------------------------------------------------------------------------
// list_workspaces / list_workspace_names
// ---------------------------------------------------------------------------

#[test]
fn list_workspaces_empty_when_no_worktrees() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    let workspaces = pipeline::list_workspaces(&ctx, guard.repo_path(), "worktrees").unwrap();
    assert!(
        workspaces.is_empty(),
        "expected no worktrees in fresh repo, got {workspaces:?}"
    );
}

#[test]
fn list_workspace_names_empty_initially() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    let names = pipeline::list_workspace_names(&ctx, guard.repo_path(), "worktrees").unwrap();
    assert!(names.is_empty());
}

// ---------------------------------------------------------------------------
// worktree lifecycle: create → list → branch → remove → verify gone
// ---------------------------------------------------------------------------

#[test]
fn worktree_full_lifecycle() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-lifecycle");
    let branch = "feat/lifecycle-test";

    // --- Create ---
    pipeline::create_workspace(&ctx, repo_root, &target, branch, "main").unwrap();

    // --- List ---
    let workspaces = pipeline::list_workspaces(&ctx, repo_root, "worktrees").unwrap();
    assert!(
        workspaces.iter().any(|p| p == &target),
        "created worktree not found in {workspaces:?}"
    );

    // --- Branch ---
    let found_branch = pipeline::workspace_branch(&ctx, &target).unwrap();
    assert_eq!(found_branch, branch);

    // --- Remove ---
    pipeline::remove_workspace(&ctx, repo_root, &target, branch).unwrap();

    // --- Verify gone ---
    let workspaces = pipeline::list_workspaces(&ctx, repo_root, "worktrees").unwrap();
    assert!(
        !workspaces.iter().any(|p| p == &target),
        "worktree should be removed, still found: {workspaces:?}"
    );
}

#[test]
fn create_multiple_worktrees_and_list_all() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    let repo_root = guard.repo_path();

    let branches = ["feat/alpha", "feat/beta", "feat/gamma"];
    let mut targets = Vec::new();

    for &b in &branches {
        let t = guard.work_dir().join(b);
        pipeline::create_workspace(&ctx, repo_root, &t, b, "main").unwrap();
        targets.push((t, b));
    }

    let workspaces = pipeline::list_workspaces(&ctx, repo_root, "worktrees").unwrap();
    for (t, _) in &targets {
        assert!(workspaces.iter().any(|p| p == t), "missing {t:?}");
    }

    // Clean up all worktrees
    for (t, b) in &targets {
        pipeline::remove_workspace(&ctx, repo_root, t, b).unwrap();
    }

    let workspaces = pipeline::list_workspaces(&ctx, repo_root, "worktrees").unwrap();
    for (t, _) in &targets {
        assert!(!workspaces.iter().any(|p| p == t), "should be gone: {t:?}");
    }
}

#[test]
fn delete_branch_removes_it() {
    let guard = init_ephemeral_repo();
    let ctx = e2e_ctx();
    let repo_root = guard.repo_path();
    let target = guard.work_dir().join("feat-to-delete");
    let branch = "feat/to-delete";

    pipeline::create_workspace(&ctx, repo_root, &target, branch, "main").unwrap();
    pipeline::remove_workspace(&ctx, repo_root, &target, branch).unwrap();

    // Delete the branch (it still exists after worktree remove)
    pipeline::delete_branch(&ctx, repo_root, branch);

    // Verify it's gone: trying to create a worktree from the same branch
    // would fail if it still existed (it would try to reuse), or we can
    // just check that it's not in `git branch --list`
    let branch_target = guard.work_dir().join("feat-to-delete-v2");
    pipeline::create_workspace(&ctx, repo_root, &branch_target, branch, "main").unwrap();

    let found = pipeline::workspace_branch(&ctx, &branch_target).unwrap();
    assert_eq!(found, branch);

    // Clean up
    pipeline::remove_workspace(&ctx, repo_root, &branch_target, branch).unwrap();
    pipeline::delete_branch(&ctx, repo_root, branch);
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

#[test]
fn dispatch_unknown_command_shows_error() {
    let ctx = e2e_ctx();
    let result = pipeline::dispatch(
        &ctx,
        &["tmux-worktrees".to_string(), "nonexistent-cmd".to_string()],
    );
    // Should succeed (error is displayed in tmux, not returned)
    assert!(result.is_ok());
}
