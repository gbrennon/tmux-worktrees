use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{LazyLock, Mutex};

use tempfile::TempDir;
use tmux_worktrees::core::ports::{GitPort, TmuxPort};
use tmux_worktrees::core::selector::SelectionResult;
use tmux_worktrees::infrastructure::git_executor::GitExecutor;
use tmux_worktrees::presentation::cli::Cli;
use tmux_worktrees::presentation::loading_port::LoadingRunner;
use tmux_worktrees::presentation::selector_port::SelectorRunner;

use super::common::fakes::loading::FakeLoading;
use super::common::fakes::selector::FakeSelector;
use super::common::fakes::tmux::FakeTmxTest;

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

struct RepoGuard {
    _dir: TempDir,
    repo: PathBuf,
}

fn init_repo() -> RepoGuard {
    let dir = TempDir::new().unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir(&repo).unwrap();
    let run = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(&repo)
            .status()
            .unwrap();
        assert!(status.success(), "git command failed: {args:?}");
    };
    run(&["init"]);
    run(&[
        "-c",
        "user.name=e2e-test",
        "-c",
        "user.email=e2e@test",
        "commit",
        "--allow-empty",
        "-m",
        "init",
    ]);
    run(&["branch", "-M", "main"]);
    RepoGuard { _dir: dir, repo }
}

#[test]
fn resume_external_worktree_uses_git_path_and_name() {
    let _env_lock = ENV_LOCK.lock().unwrap();
    let guard = init_repo();
    let external = guard._dir.path().join("external-worktree");
    let status = Command::new("git")
        .args([
            "worktree",
            "add",
            external.to_str().unwrap(),
            "-b",
            "feat/external",
            "main",
        ])
        .current_dir(&guard.repo)
        .status()
        .unwrap();
    assert!(status.success());

    let previous_directory = std::env::current_dir().unwrap();
    let previous_root = std::env::var_os("TMUX_WORKTREES_ROOT");
    unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", &guard.repo) };

    let tmux = FakeTmxTest::new(&guard.repo.to_string_lossy());
    let selected_window = tmux.selected_window.clone();
    let selector = FakeSelector::new(vec![SelectionResult::Selected(0)]);
    let cli = cli_with(tmux, GitExecutor::default(), selector, FakeLoading::new());
    let names = cli.list_workspace_names(&guard.repo, "worktrees").unwrap();
    let result = cli.run_choose();

    match previous_root {
        Some(root) => unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root) },
        None => unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") },
    }
    std::env::set_current_dir(previous_directory).unwrap();

    assert!(result.is_ok(), "expected Ok, got {result:?}");
    assert_eq!(names, vec!["external-worktree"]);
    assert_eq!(
        selected_window.borrow().as_ref().map(|(_, path, _)| path),
        Some(&external.to_string_lossy().into_owned())
    );
}
