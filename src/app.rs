use anyhow::{Context, Result};
use fuzzy_matcher::{clangd::ClangdMatcher, FuzzyMatcher};
use std::path::Path;

use crate::core::{
    merge_checker::MergeChecker,
    ports::{GitPort, TmuxPort},
    project_locator::ProjectLocator,
    selector::{SelectionResult, Selector},
    workspace_resolver::WorkspaceResolver,
};

// ---------------------------------------------------------------------------
// Routing
// ---------------------------------------------------------------------------

pub fn dispatch(tmux: &dyn TmuxPort, git: &dyn GitPort, args: &[String]) -> Result<()> {
    let (cmd, rest) = parse_args(args);
    match cmd.as_str() {
        "choose" => run_choose(tmux, git),
        "create-worktree" => run_create(
            tmux,
            git,
            rest.get(1).map(String::as_str).unwrap_or_default(),
        ),
        "cleanup" => run_cleanup(tmux, git),
        other => {
            tmux.show_error(&format!("Unknown command: {other}"))?;
            Ok(())
        }
    }
}

pub fn parse_args(args: &[String]) -> (String, Vec<String>) {
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
    (
        rest.first()
            .cloned()
            .unwrap_or_else(|| "choose".to_string()),
        rest,
    )
}

pub fn spawn_in_popup(tmux: &dyn TmuxPort, _git: &dyn GitPort, args: &[String]) -> Result<()> {
    let root = match find_repo_root(tmux) {
        Ok(r) => r,
        Err(e) => {
            tmux.show_error(&format!("{e:#}"))?;
            return Ok(());
        }
    };
    let exe = std::env::current_exe().context("Failed to resolve current executable")?;
    let mut cmd = crate::utils::shell_quote(&exe.to_string_lossy());
    for a in args.iter().skip(1) {
        if a.starts_with("--root") {
            continue;
        }
        cmd.push(' ');
        cmd.push_str(&crate::utils::shell_quote(a));
    }
    cmd.push_str(" --root=");
    cmd.push_str(&crate::utils::shell_quote(&root));
    let _ = tmux.display_popup("60%", "50%", &root, &cmd);
    Ok(())
}

// ---------------------------------------------------------------------------
// Choose
// ---------------------------------------------------------------------------

pub fn run_choose(tmux: &dyn TmuxPort, git: &dyn GitPort) -> Result<()> {
    let repo_root = find_repo_root(tmux).context("Failed to find repository root")?;
    let repo_root_path = Path::new(&repo_root);
    std::env::set_current_dir(repo_root_path)?;
    let worktree_dir = tmux.resolve_workspace_dir();
    let existing = list_workspace_names(git, repo_root_path, &worktree_dir)?;
    let header = if existing.is_empty() {
        "No workspaces yet — type a name and press Enter to create"
    } else {
        "Type to filter, Enter to select/create"
    };
    let mut selector = Selector::new(existing.clone(), true);
    let filtered = selector.filter("");
    let picked = run_selector(&mut selector, &filtered, "Workspace> ", header)?;
    let branch = match picked {
        SelectionResult::Selected(i) => existing[i].clone(),
        SelectionResult::Custom(q) => q,
        SelectionResult::Cancelled => return Ok(()),
    };
    let ws = repo_root_path.join(&worktree_dir).join(&branch);
    if ws.is_dir() {
        if let Ok(resolved) = workspace_branch(git, &ws) {
            if !resolved.is_empty() && resolved != "?" {
                return run_create(tmux, git, &resolved);
            }
        }
    }
    run_create(tmux, git, &branch)
}

fn run_selector(
    selector: &mut Selector,
    _filtered: &[usize],
    prompt: &str,
    header: &str,
) -> Result<SelectionResult> {
    use ratatui::{
        layout::{Constraint, Direction, Layout},
        style::{Color, Style},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    };
    let mut terminal = setup_terminal()?;
    terminal.clear().context("Failed to clear terminal")?;
    let matcher = ClangdMatcher::default();

    let result = loop {
        let filtered: Vec<usize> = selector
            .items()
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if selector.query().is_empty() {
                    Some(i)
                } else {
                    matcher.fuzzy_match(s, selector.query()).map(|_| i)
                }
            })
            .collect();

        selector.clamp_selection(filtered.len());

        terminal
            .draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(area);
                let input = Paragraph::new(format!("{prompt}{}", selector.query()))
                    .block(Block::default().borders(Borders::ALL));
                let hint = Paragraph::new(header).style(Style::default().fg(Color::DarkGray));
                f.render_widget(input, chunks[0]);
                f.render_widget(hint, chunks[1]);

                let list_items: Vec<ListItem> = filtered
                    .iter()
                    .map(|&i| ListItem::new(selector.items()[i].as_str()))
                    .collect();
                let mut ls = ListState::default();
                ls.select(Some(selector.selected_index()));
                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(list, chunks[2], &mut ls);
            })
            .context("Failed to draw terminal")?;

        if let crossterm::event::Event::Key(k) =
            crossterm::event::read().context("Failed to read key event")?
        {
            if let Some(result) = selector.process_key((k.code, k.modifiers), &filtered) {
                break result;
            }
        }
    };

    restore_terminal(&mut terminal)?;
    Ok(result)
}

// ---------------------------------------------------------------------------
// Create
// ---------------------------------------------------------------------------

pub fn run_create(tmux: &dyn TmuxPort, git: &dyn GitPort, branch: &str) -> Result<()> {
    if branch.trim().is_empty() {
        tmux.show_error(
            "Branch name cannot be empty — type a name (e.g. feat/foo) or select an existing workspace",
        )?;
        return Ok(());
    }
    let repo_root = find_repo_root(tmux).context("Failed to find repository root")?;
    let repo_root_path = Path::new(&repo_root);
    std::env::set_current_dir(repo_root_path)?;
    let worktree_dir = tmux.resolve_workspace_dir();
    let project_locator = ProjectLocator;
    project_locator.ensure_workspace_directory(repo_root_path, &worktree_dir)?;
    let target = WorkspaceResolver.resolve_path(repo_root_path, &worktree_dir, branch);
    if !target.is_dir() {
        let default_branch = resolve_default_branch(git, repo_root_path)?;
        if let Err(e) = create_workspace(git, repo_root_path, &target, branch, &default_branch) {
            tmux.show_error(&format!("Workspace creation failed:\n\n{e:#}"))?;
            return Ok(());
        }
    }
    let target_abs = WorkspaceResolver.resolve_absolute(&target);
    let command = tmux.resolve_shell_command();
    tmux.select_or_create_window(branch, &target_abs, &command)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Cleanup
// ---------------------------------------------------------------------------

pub fn run_cleanup(tmux: &dyn TmuxPort, git: &dyn GitPort) -> Result<()> {
    let repo_root = find_repo_root(tmux).context("Failed to find repository root")?;
    let repo_root_path = Path::new(&repo_root);
    std::env::set_current_dir(repo_root_path)?;
    let worktree_dir = tmux.resolve_workspace_dir();
    let default_branch = resolve_default_branch(git, repo_root_path)?;
    let auto_fetch = tmux
        .get_option("worktree-auto-fetch")
        .unwrap_or_else(|| "true".to_string());
    if auto_fetch != "false" {
        let _ = git.run_in(
            repo_root_path,
            &["fetch", "origin", &default_branch, "--no-tags"],
        );
    }
    let workspaces = list_workspaces(git, repo_root_path, &worktree_dir)?;
    let mut items = Vec::new();
    let mut paths = Vec::new();
    let merge_checker = MergeChecker;
    for ws in &workspaces {
        let branch = workspace_branch(git, ws).unwrap_or_else(|_| "?".to_string());
        let merged = merge_checker
            .is_merged(ws, &default_branch, |dir, remote| {
                git.run_in(dir, &["merge-base", "--is-ancestor", "HEAD", remote])
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
        tmux.show_error(&format!(
            "No workspaces found in {worktree_dir}/ — press Enter to dismiss"
        ))?;
        return Ok(());
    }
    let items_clone = items.clone();
    let mut selector = Selector::new(items, false);
    let filtered = selector.filter("");
    let picked = run_selector(
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
    if let Err(e) = remove_workspace(tmux, git, repo_root_path, ws_dir, &branch) {
        tmux.show_error(&format!("Workspace removal failed:\n\n{e:#}"))?;
        return Ok(());
    }
    delete_branch(git, repo_root_path, &branch);
    let _ = tmux.run(&["display-message", &format!("Removed workspace: {branch}")]);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn find_repo_root(tmux: &dyn TmuxPort) -> Result<String> {
    let project_locator = ProjectLocator;
    if let Some(root) = ProjectLocator::from_env() {
        return Ok(root);
    }
    if let Some(root) = project_locator.by_walking(&std::env::current_dir()?) {
        return Ok(root);
    }
    if let Ok(Some(v)) = tmux.show_environment("MAIN_PROJECT_PATH") {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    let dir = tmux.current_pane_path()?;
    let project_locator = ProjectLocator;
    if let Some(root) = project_locator.by_walking(Path::new(&dir)) {
        return Ok(root);
    }
    anyhow::bail!("Not in a git repository")
}

pub fn resolve_default_branch(git: &dyn GitPort, repo_root: &Path) -> Result<String> {
    let project_locator = ProjectLocator;
    let global = git
        .run_in(repo_root, &["config", "--global", "init.defaultBranch"])
        .ok()
        .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
        .unwrap_or_default();
    let local = git
        .run_in(repo_root, &["config", "init.defaultBranch"])
        .ok()
        .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
        .unwrap_or_default();
    let current = git
        .run_in(repo_root, &["branch", "--show-current"])
        .ok()
        .and_then(|(s, o, _)| if s == 0 { Some(o) } else { None })
        .unwrap_or_default();
    Ok(project_locator.determine_default_branch(&global, &local, &current))
}

pub fn list_workspaces(
    git: &dyn GitPort,
    repo_root: &Path,
    work_dir: &str,
) -> Result<Vec<std::path::PathBuf>> {
    let (status, out, _) = git.run_in(repo_root, &["worktree", "list", "--porcelain"])?;
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

pub fn list_workspace_names(
    git: &dyn GitPort,
    repo_root: &Path,
    work_dir: &str,
) -> Result<Vec<String>> {
    let mut names: Vec<String> = list_workspaces(git, repo_root, work_dir)?
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    names.sort();
    Ok(names)
}

pub fn create_workspace(
    git: &dyn GitPort,
    repo_root: &Path,
    target: &Path,
    branch: &str,
    base: &str,
) -> Result<()> {
    let _ = git.silent_in(repo_root, &["fetch", "origin", "--quiet"]);
    let mut base = base.to_string();
    let remote_base = format!("refs/remotes/origin/{base}");
    let exists = git
        .run_in(
            repo_root,
            &["show-ref", "--verify", "--quiet", &remote_base],
        )
        .map(|(s, _, _)| s == 0)
        .unwrap_or(false);
    if exists {
        base = format!("origin/{base}");
    }
    let (status, stdout, stderr) = git.run_in(
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
        anyhow::bail!("{}", if stdout.is_empty() { stderr } else { stdout });
    }
    Ok(())
}

pub fn remove_workspace(
    tmux: &dyn TmuxPort,
    git: &dyn GitPort,
    repo_root: &Path,
    ws_dir: &Path,
    branch: &str,
) -> Result<()> {
    if !branch.is_empty() {
        tmux.kill_window(branch)?;
    }
    let (status, stdout, stderr) = git.run_in(
        repo_root,
        &["worktree", "remove", "--force", ws_dir.to_str().unwrap()],
    )?;
    if status != 0 {
        anyhow::bail!("{}", if stdout.is_empty() { stderr } else { stdout });
    }
    Ok(())
}

pub fn workspace_branch(git: &dyn GitPort, ws_dir: &Path) -> Result<String> {
    let (status, out, _) = git.run_in(ws_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    if status != 0 {
        return Ok("?".to_string());
    }
    Ok(out)
}

pub fn delete_branch(git: &dyn GitPort, repo_root: &Path, branch: &str) {
    let _ = git.silent_in(repo_root, &["branch", "-D", branch]);
}

// ---------------------------------------------------------------------------
// Terminal helpers
// ---------------------------------------------------------------------------

pub fn setup_terminal(
) -> Result<ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>> {
    use std::io::stdout;
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    ratatui::Terminal::new(backend).context("Failed to initialize terminal")
}

fn restore_terminal(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_defaults_to_choose() {
        let (cmd, rest) = parse_args(&["tmux-worktrees".to_string()]);
        assert_eq!(cmd, "choose");
        assert!(rest.is_empty());
    }

    #[test]
    fn parse_args_extracts_root_and_branch() {
        let (cmd, rest) = parse_args(&[
            "tmux-worktrees".to_string(),
            "create-worktree".to_string(),
            "feat/foo".to_string(),
            "--root=/home/user/repo".to_string(),
        ]);
        assert_eq!(cmd, "create-worktree");
        assert_eq!(rest, vec!["create-worktree", "feat/foo"]);
    }

    #[test]
    fn parse_args_supports_separate_root_flag() {
        let (cmd, rest) = parse_args(&[
            "tmux-worktrees".to_string(),
            "choose".to_string(),
            "--root".to_string(),
            "/home/user/repo".to_string(),
        ]);
        assert_eq!(cmd, "choose");
        assert_eq!(rest, vec!["choose"]);
    }

    #[test]
    fn dispatch_unknown_command_shows_error() {
        use crate::infrastructure::command_runner::test_support::FakeRunner;
        use crate::infrastructure::git_executor::GitExecutor;
        use crate::infrastructure::tmux_executor::TmuxExecutor;

        let f = FakeRunner::new();
        f.ok(
            "tmux",
            "display-message:-d:5000:tmux-worktrees: Unknown command: bogus",
            0,
            "",
            "",
        );

        let tmux = TmuxExecutor::with_runner(Box::new(f));
        let git = GitExecutor::default();
        let args = vec!["tmux-worktrees".to_string(), "bogus".to_string()];
        dispatch(&tmux, &git, &args).unwrap();
    }
}
