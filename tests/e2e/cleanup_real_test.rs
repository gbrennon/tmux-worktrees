use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tempfile::TempDir;

struct RepoGuard {
    _dir: TempDir,
    root: PathBuf,
}

impl RepoGuard {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        std::fs::create_dir(&root).unwrap();
        run_command("git", &["init", "-q", "-b", "main"], Some(&root));
        run_command(
            "git",
            &["config", "user.email", "e2e@example.invalid"],
            Some(&root),
        );
        run_command("git", &["config", "user.name", "e2e"], Some(&root));
        std::fs::write(root.join("README"), "e2e\n").unwrap();
        run_command("git", &["add", "README"], Some(&root));
        run_command("git", &["commit", "-qm", "initial"], Some(&root));
        std::fs::create_dir(root.join(".worktrees")).unwrap();
        for (branch, directory) in [
            ("feat/e2e-a", ".worktrees/feat-e2e-a"),
            ("feat/e2e-b", ".worktrees/feat-e2e-b"),
        ] {
            run_command(
                "git",
                &["worktree", "add", "-q", "-b", branch, directory],
                Some(&root),
            );
        }
        Self { _dir: dir, root }
    }
}

struct TmuxGuard {
    socket: String,
}

impl TmuxGuard {
    fn new(root: &Path, binary: &Path) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let socket = format!("tmux-worktrees-e2e-{}-{nonce}", std::process::id());
        let _ = tmux_command(&socket, &["kill-server"]);
        let root = root.to_str().unwrap();
        let binary = binary.to_str().unwrap();
        let output = tmux_command(
            &socket,
            &[
                "new-session",
                "-d",
                "-s",
                "cleanup",
                "-c",
                root,
                binary,
                "cleanup",
                "--root",
                root,
            ],
        );
        assert_success(&output, "start cleanup tmux session");
        Self { socket }
    }

    fn send_key(&self, key: &str) {
        let output = tmux_command(&self.socket, &["send-keys", "-t", "cleanup:0", key]);
        assert_success(&output, "send tmux key");
    }

    fn capture(&self) -> String {
        let output = tmux_command(&self.socket, &["capture-pane", "-p", "-t", "cleanup:0"]);
        assert_success(&output, "capture cleanup pane");
        String::from_utf8(output.stdout).unwrap()
    }

    fn wait_for(&self, expected: &str) -> String {
        for _ in 0..100 {
            let pane = self.capture();
            if pane.contains(expected) {
                return pane;
            }
            sleep(Duration::from_millis(50));
        }
        panic!(
            "cleanup pane never contained {expected:?}:\n{}",
            self.capture()
        );
    }
}

impl Drop for TmuxGuard {
    fn drop(&mut self) {
        let _ = tmux_command(&self.socket, &["kill-server"]);
    }
}

fn binary_path() -> PathBuf {
    [
        "CARGO_BIN_EXE_tmux-worktrees",
        "CARGO_BIN_EXE_tmux_worktrees",
    ]
    .into_iter()
    .find_map(std::env::var_os)
    .map(PathBuf::from)
    .expect("Cargo did not provide the tmux-worktrees binary path")
}

fn tmux_command(socket: &str, args: &[&str]) -> Output {
    Command::new("tmux")
        .args(["-L", socket, "-f", "/dev/null"])
        .args(args)
        .output()
        .unwrap()
}

fn run_command(program: &str, args: &[&str], cwd: Option<&Path>) -> Output {
    let mut command = Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let output = command.output().unwrap();
    assert_success(&output, program);
    output
}

fn assert_success(output: &Output, action: &str) {
    assert!(
        output.status.success(),
        "{action} failed: status={} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn binary_entrypoint_removes_two_worktrees_after_confirmation() {
    let repo = RepoGuard::new();
    let binary = binary_path();
    let tmux = TmuxGuard::new(&repo.root, &binary);

    let initial = tmux.wait_for("Space to toggle, Enter to continue, Esc to cancel");
    assert!(initial.contains("[ ] ✗ active  | feat/e2e-a"));
    assert!(initial.contains("[ ] ✗ active  | feat/e2e-b"));

    tmux.send_key("Space");
    tmux.send_key("Down");
    tmux.send_key("Space");
    tmux.send_key("Enter");
    let confirmation = tmux.wait_for("Confirm removal>");
    assert!(confirmation.contains("Remove 2 worktree(s): feat/e2e-a, feat/e2e-b"));

    tmux.send_key("Enter");

    for _ in 0..100 {
        let worktrees = run_command(
            "git",
            &["worktree", "list", "--porcelain"],
            Some(&repo.root),
        );
        let branches = run_command(
            "git",
            &["branch", "--format=%(refname:short)"],
            Some(&repo.root),
        );
        let worktrees = String::from_utf8(worktrees.stdout).unwrap();
        let branches = String::from_utf8(branches.stdout).unwrap();
        if !worktrees.contains("feat-e2e-a")
            && !worktrees.contains("feat-e2e-b")
            && !branches.contains("feat/e2e-a")
            && !branches.contains("feat/e2e-b")
        {
            return;
        }
        sleep(Duration::from_millis(50));
    }

    panic!(
        "selected worktrees or branches remained:\n{}\n{}",
        String::from_utf8_lossy(
            &run_command(
                "git",
                &["worktree", "list", "--porcelain"],
                Some(&repo.root)
            )
            .stdout
        ),
        String::from_utf8_lossy(
            &run_command(
                "git",
                &["branch", "--format=%(refname:short)"],
                Some(&repo.root),
            )
            .stdout
        )
    );
}
