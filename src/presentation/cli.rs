use crate::core::error::Result;
use crate::presentation::loading_port::LoadingRunner;
use crate::presentation::selector_port::SelectorRunner;
use std::io::IsTerminal;
use std::path::Path;

use super::command::Command;
use crate::core::{
    merge_checker::MergeChecker,
    ports::{GitPort, TmuxPort},
    project_locator::ProjectLocator,
    selector::{SelectionResult, Selector},
    workspace_resolver::WorkspaceResolver,
};

/// Presentation-layer CLI — owns the tmux, git, selector and loading adapters
/// and exposes one method per user-facing command.
pub struct Cli {
    tmux: Box<dyn TmuxPort>,
    git: Box<dyn GitPort>,
    selector: Box<dyn SelectorRunner>,
    loading: Box<dyn LoadingRunner>,
}

impl Cli {
    pub fn new(
        tmux: Box<dyn TmuxPort>,
        git: Box<dyn GitPort>,
        selector: Box<dyn SelectorRunner>,
        loading: Box<dyn LoadingRunner>,
    ) -> Self {
        Self {
            tmux,
            git,
            selector,
            loading,
        }
    }

    /// Entry point: parse the command-line and dispatch.
    pub fn run(&self, args: &[String]) {
        let (cmd, rest) = self.parse_args(args);

        let result = if cmd.is_interactive() && !std::io::stdin().is_terminal() {
            match self.spawn_in_popup(args) {
                Ok(()) => Ok(()),
                Err(_) => self.dispatch(cmd, &rest),
            }
        } else {
            self.dispatch(cmd, &rest)
        };

        if let Err(e) = result {
            let _ = self.tmux.show_error(&format!("{e:#}"));
        }
    }

    pub fn dispatch(&self, cmd: Command, rest: &[String]) -> Result<()> {
        match cmd {
            Command::Choose => self.run_choose(),
            Command::CreateWorktree => {
                self.run_create(rest.first().map(String::as_str).unwrap_or_default())
            }
            Command::Cleanup => self.run_cleanup(),
            Command::DemoLoading => self.run_demo_loading(),
        }
    }

    pub fn parse_args(&self, args: &[String]) -> (Command, Vec<String>) {
        let mut rest = Vec::new();
        let mut i = 1;
        while i < args.len() {
            let a = &args[i];
            if let Some(root) = a.strip_prefix("--root=") {
                unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root) };
            } else if a == "--root" {
                if let Some(root) = args.get(i + 1) {
                    unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", root) };
                    i += 1;
                }
            } else {
                rest.push(a.clone());
            }
            i += 1;
        }
        let cmd = rest
            .first()
            .map(|s| s.parse::<Command>().unwrap_or(Command::Choose))
            .unwrap_or(Command::Choose);
        if !rest.is_empty() {
            rest.remove(0);
        }
        (cmd, rest)
    }

    pub fn spawn_in_popup(&self, args: &[String]) -> Result<()> {
        let root = match self.find_repo_root() {
            Ok(r) => r,
            Err(e) => {
                self.tmux.show_error(&format!("{e:#}"))?;
                return Ok(());
            }
        };
        let exe = std::env::current_exe().unwrap_or_else(|_| {
            args.first()
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from("tmux-worktrees"))
        });
        let mut cmd = crate::utils::ShellQuoter::quote(&exe.to_string_lossy());
        for a in args.iter().skip(1) {
            if a.starts_with("--root") {
                continue;
            }
            cmd.push(' ');
            cmd.push_str(&crate::utils::ShellQuoter::quote(a));
        }
        cmd.push_str(" --root=");
        cmd.push_str(&crate::utils::ShellQuoter::quote(&root));
        let _ = self.tmux.display_popup("60%", "50%", &root, &cmd);
        Ok(())
    }

    pub fn run_choose(&self) -> Result<()> {
        let repo_root = self.find_repo_root().map_err(|e| {
            crate::core::error::Error::new(format!("Failed to find repository root: {}", e))
        })?;
        let repo_root_path = Path::new(&repo_root);
        std::env::set_current_dir(repo_root_path)?;
        let worktree_dir = self.tmux.resolve_workspace_dir();
        let existing = self.list_workspace_names(repo_root_path, &worktree_dir)?;
        let header = if existing.is_empty() {
            "No workspaces yet — type a name and press Enter to create"
        } else {
            "Type to filter, Enter to select/create"
        };
        let mut selector = Selector::new(existing.clone(), true);
        let filtered = selector.filter("");
        let picked = self
            .selector
            .run_selector(&mut selector, &filtered, "Workspace> ", header)?;
        let branch = match picked {
            SelectionResult::Selected(i) => existing[i].clone(),
            SelectionResult::Custom(q) => q,
            SelectionResult::Cancelled => return Ok(()),
        };
        let ws = repo_root_path.join(&worktree_dir).join(&branch);
        if ws.is_dir()
            && let Ok(resolved) = self.workspace_branch(&ws)
            && !resolved.is_empty()
            && resolved != "?"
        {
            return self.run_create(&resolved);
        }
        self.run_create(&branch)
    }

    pub fn run_create(&self, branch: &str) -> Result<()> {
        if branch.trim().is_empty() {
            self.tmux.show_error(
                "Branch name cannot be empty — type a name (e.g. feat/foo) or select an existing workspace",
            )?;
            return Ok(());
        }
        let repo_root = self.find_repo_root().map_err(|e| {
            crate::core::error::Error::new(format!("Failed to find repository root: {}", e))
        })?;
        let repo_root_path = Path::new(&repo_root);
        std::env::set_current_dir(repo_root_path)?;
        let worktree_dir = self.tmux.resolve_workspace_dir();
        let project_locator = ProjectLocator;
        project_locator.ensure_workspace_directory(repo_root_path, &worktree_dir)?;
        let target = WorkspaceResolver.resolve_path(repo_root_path, &worktree_dir, branch);
        if !target.is_dir() {
            let default_branch = self.resolve_default_branch(repo_root_path)?;
            if let Err(e) = self.create_workspace(repo_root_path, &target, branch, &default_branch)
            {
                self.tmux
                    .show_error(&format!("Workspace creation failed:\n\n{e:#}"))?;
                return Ok(());
            }
        }
        let target_abs = WorkspaceResolver.resolve_absolute(&target);
        let command = self.tmux.resolve_shell_command();
        self.tmux
            .select_or_create_window(branch, &target_abs, &command)?;
        Ok(())
    }

    pub fn run_cleanup(&self) -> Result<()> {
        let repo_root = self.find_repo_root().map_err(|e| {
            crate::core::error::Error::new(format!("Failed to find repository root: {}", e))
        })?;
        let repo_root_path = Path::new(&repo_root);
        std::env::set_current_dir(repo_root_path)?;
        let worktree_dir = self.tmux.resolve_workspace_dir();
        let default_branch = self.resolve_default_branch(repo_root_path)?;
        let auto_fetch = self
            .tmux
            .get_option("worktree-auto-fetch")
            .unwrap_or_else(|| "true".to_string());
        if auto_fetch != "false" {
            let git = &self.git;
            let loading = &self.loading;
            let fetch_branch = default_branch.clone();
            let _ = loading.run_loading(
                "Fetching default branch…",
                Box::new(move || {
                    git.run_in(
                        repo_root_path,
                        &["fetch", "origin", &fetch_branch, "--no-tags"],
                    )
                    .map(|_| ())
                }),
            );
        }
        let workspaces = self.list_workspaces(repo_root_path, &worktree_dir)?;
        let mut items = Vec::new();
        let mut paths = Vec::new();
        let merge_checker = MergeChecker;
        for ws in &workspaces {
            let branch = self
                .workspace_branch(ws)
                .unwrap_or_else(|_| "?".to_string());
            let merged = merge_checker
                .is_merged(ws, &default_branch, |dir, remote| {
                    self.git
                        .run_in(dir, &["merge-base", "--is-ancestor", "HEAD", remote])
                        .map(|(s, _, _)| s == 0)
                        .map_err(|e| format!("{e:#}"))
                })
                .unwrap_or(false);
            let item = if merged {
                format!("✓ merged  | {branch}")
            } else {
                format!("✗ active  | {branch}")
            };
            items.push(item);
            paths.push(ws.clone());
        }
        if items.is_empty() {
            self.tmux.show_error(&format!(
                "No workspaces found in {worktree_dir}/ — press Enter to dismiss"
            ))?;
            return Ok(());
        }
        let items_clone = items.clone();
        let mut selector = Selector::new(items, false);
        let filtered = selector.filter("");
        let picked = self.selector.run_selector(
            &mut selector,
            &filtered,
            "Remove workspace> ",
            "Enter to remove selected workspace",
        )?;
        let idx = match picked {
            SelectionResult::Selected(i) => i,
            _ => return Ok(()),
        };
        let ws_dir = &paths[idx];
        let branch = WorkspaceResolver
            .extract_branch_from_display(&items_clone[idx])
            .unwrap_or_default();
        if let Err(e) = self.remove_workspace(repo_root_path, ws_dir, &branch) {
            self.tmux
                .show_error(&format!("Workspace removal failed:\n\n{e:#}"))?;
            return Ok(());
        }
        self.delete_branch(repo_root_path, &branch);
        let _ = self
            .tmux
            .run(&["display-message", &format!("Removed workspace: {branch}")]);
        Ok(())
    }

    /// Show the loading indicator for 3 seconds — purely for visual verification.
    pub fn run_demo_loading(&self) -> Result<()> {
        self.loading.run_loading(
            "Demo: loading indicator test…",
            Box::new(|| {
                std::thread::sleep(std::time::Duration::from_secs(3));
                Ok(())
            }),
        )
    }

    pub fn find_repo_root(&self) -> Result<String> {
        self.find_repo_root_from(&std::env::current_dir()?)
    }

    pub fn find_repo_root_from(&self, start: &Path) -> Result<String> {
        let project_locator = ProjectLocator;
        if let Some(root) = ProjectLocator::from_env() {
            return Ok(root);
        }
        if let Some(root) = project_locator.by_walking(start) {
            return Ok(root);
        }
        if let Ok(Some(v)) = self.tmux.show_environment("MAIN_PROJECT_PATH")
            && !v.is_empty()
        {
            return Ok(v);
        }
        let dir = self.tmux.current_pane_path()?;
        let project_locator = ProjectLocator;
        if let Some(root) = project_locator.by_walking(Path::new(&dir)) {
            return Ok(root);
        }
        Err(crate::core::error::Error::new("Not in a git repository"))
    }
    pub fn resolve_default_branch(&self, repo_root: &Path) -> Result<String> {
        let project_locator = ProjectLocator;
        let global = self
            .git
            .run_in(repo_root, &["config", "--global", "init.defaultBranch"])
            .ok()
            .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
            .unwrap_or_default();
        let local = self
            .git
            .run_in(repo_root, &["config", "init.defaultBranch"])
            .ok()
            .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
            .unwrap_or_default();
        let current = self
            .git
            .run_in(repo_root, &["branch", "--show-current"])
            .ok()
            .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
            .unwrap_or_default();
        Ok(project_locator.determine_default_branch(&global, &local, &current))
    }

    pub fn list_workspaces(
        &self,
        repo_root: &Path,
        work_dir: &str,
    ) -> Result<Vec<std::path::PathBuf>> {
        let (status, out, _) = self
            .git
            .run_in(repo_root, &["worktree", "list", "--porcelain"])?;
        if status != 0 {
            return Ok(Vec::new());
        }
        let prefix = std::fs::canonicalize(repo_root.join(work_dir))
            .unwrap_or_else(|_| repo_root.join(work_dir));
        let mut res = Vec::new();
        for line in out.lines() {
            if let Some(p) = line.strip_prefix("worktree ") {
                let p = std::path::PathBuf::from(p.trim());
                let matches = p.starts_with(&prefix)
                    || std::fs::canonicalize(&p)
                        .map(|c| c.starts_with(&prefix))
                        .unwrap_or(false);
                if matches {
                    res.push(p);
                }
            }
        }
        Ok(res)
    }

    pub fn list_workspace_names(&self, repo_root: &Path, work_dir: &str) -> Result<Vec<String>> {
        let mut names: Vec<String> = self
            .list_workspaces(repo_root, work_dir)?
            .iter()
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
            .collect();
        names.sort();
        Ok(names)
    }

    pub fn create_workspace(
        &self,
        repo_root: &Path,
        target: &Path,
        branch: &str,
        base: &str,
    ) -> Result<()> {
        let git = &self.git;
        let loading = &self.loading;
        let _ = loading.run_loading(
            "Fetching origin…",
            Box::new(move || git.silent_in(repo_root, &["fetch", "origin", "--quiet"])),
        );
        let mut base = base.to_string();
        let remote_base = format!("refs/remotes/origin/{base}");
        let exists = self
            .git
            .run_in(
                repo_root,
                &["show-ref", "--verify", "--quiet", &remote_base],
            )
            .map(|(s, _, _)| s == 0)
            .unwrap_or(false);
        if exists {
            base = format!("origin/{base}");
        }
        let (status, stdout, stderr) = self.git.run_in(
            repo_root,
            &[
                "worktree",
                "add",
                target.to_str().unwrap(),
                "-b",
                branch,
                &base,
            ],
        )?;
        if status != 0 {
            return Err(crate::core::error::Error::new(if stdout.is_empty() {
                stderr
            } else {
                stdout
            }));
        }
        Ok(())
    }

    pub fn remove_workspace(&self, repo_root: &Path, ws_dir: &Path, branch: &str) -> Result<()> {
        if !branch.is_empty() {
            self.tmux.kill_window(branch)?;
        }
        let (status, stdout, stderr) = self.git.run_in(
            repo_root,
            &["worktree", "remove", "--force", ws_dir.to_str().unwrap()],
        )?;
        if status != 0 {
            return Err(crate::core::error::Error::new(if stdout.is_empty() {
                stderr
            } else {
                stdout
            }));
        }
        Ok(())
    }

    pub fn workspace_branch(&self, ws_dir: &Path) -> Result<String> {
        let (status, out, _) = self
            .git
            .run_in(ws_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
        if status != 0 {
            return Ok("?".to_string());
        }
        Ok(out)
    }

    pub fn delete_branch(&self, repo_root: &Path, branch: &str) {
        let _ = self.git.silent_in(repo_root, &["branch", "-D", branch]);
    }
}
