use anyhow::{Context, Result};
use std::io::IsTerminal;
use std::path::Path;
use std::process;

mod git;
mod merge;
mod repo;
mod select;
mod tmux;
mod worktree;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(String::as_str).unwrap_or("choose");
    let interactive = matches!(cmd, "choose" | "cleanup");

    let result = if interactive && !std::io::stdin().is_terminal() {
        match spawn_in_popup(&args) {
            Ok(()) => Ok(()),
            Err(_) => dispatch(&args),
        }
    } else {
        dispatch(&args)
    };

    if let Err(e) = result {
        let _ = tmux::show_error(&format!("{e:#}"));
    }
    process::exit(0);
}

fn dispatch(args: &[String]) -> Result<()> {
    let (cmd, rest) = parse_args(args);
    match cmd.as_str() {
        "choose" => run_choose(),
        "create-worktree" => {
            run_create(rest.get(1).map(String::as_str).unwrap_or_default())
        }
        "cleanup" => run_cleanup(),
        other => {
            tmux::show_error(&format!("Unknown command: {other}"))?;
            Ok(())
        }
    }
}

fn parse_args(args: &[String]) -> (String, Vec<String>) {
    let mut rest = Vec::new();
    let mut i = 1;
    while i < args.len() {
        let a = &args[i];
        if let Some(root) = a.strip_prefix("--root=") {
            std::env::set_var("TMUX_WORKTREES_ROOT", root);
        } else if a == "--root" {
            if let Some(root) = args.get(i + 1) {
                std::env::set_var("TMUX_WORKTREES_ROOT", root);
                i += 1;
            }
        } else {
            rest.push(a.clone());
        }
        i += 1;
    }
    (rest.first().cloned().unwrap_or_else(|| "choose".to_string()), rest)
}

fn spawn_in_popup(args: &[String]) -> Result<()> {
    let root = match repo::find_repo_root() {
        Ok(r) => r,
        Err(e) => {
            tmux::show_error(&format!("{e:#}"))?;
            return Ok(());
        }
    };
    let exe = std::env::current_exe().context("Failed to resolve current executable")?;
    let mut cmd = shell_quote(&exe.to_string_lossy());
    for a in args.iter().skip(1) {
        if a.starts_with("--root") {
            continue;
        }
        cmd.push(' ');
        cmd.push_str(&shell_quote(a));
    }
    cmd.push_str(" --root=");
    cmd.push_str(&shell_quote(&root));
    let _ = tmux::run(&["display-popup", "-E", "-w", "60%", "-h", "50%", "-d", &root, &cmd]);
    Ok(())
}

fn run_choose() -> Result<()> {
    let repo_root = repo::find_repo_root().context("Failed to find repository root")?;
    std::env::set_current_dir(&repo_root)?;
    let worktree_dir = tmux::resolve_worktree_dir();
    let existing = worktree::list_worktree_names(Path::new(&repo_root), &worktree_dir)?;
    let header = if existing.is_empty() {
        "No worktrees yet — type a name and press Enter to create"
    } else {
        "Type to filter, Enter to select/create"
    };
    let picked = select::fzf_pick(&existing, "Worktree> ", header, true)?;
    let branch = match picked {
        select::Choice::Pick(i) => existing[i].clone(),
        select::Choice::Type(q) => q,
        select::Choice::Cancel => return Ok(()),
    };
    let wt = Path::new(&repo_root).join(&worktree_dir).join(&branch);
    if wt.is_dir() {
        if let Ok(resolved) = worktree::branch(&wt) {
            if !resolved.is_empty() && resolved != "?" {
                return run_create(&resolved);
            }
        }
    }
    run_create(&branch)
}

fn run_create(branch: &str) -> Result<()> {
    if branch.trim().is_empty() {
        tmux::show_error(
            "Branch name cannot be empty — type a name (e.g. feat/foo) or select an existing worktree",
        )?;
        return Ok(());
    }
    let repo_root = repo::find_repo_root().context("Failed to find repository root")?;
    std::env::set_current_dir(&repo_root)?;
    let worktree_dir = tmux::resolve_worktree_dir();
    repo::ensure_worktrees_dir(Path::new(&repo_root), &worktree_dir)?;
    let target_name = branch.replace('/', "-");
    let target = Path::new(&repo_root).join(&worktree_dir).join(&target_name);
    if !target.is_dir() {
        let default_branch = repo::resolve_default_branch(Path::new(&repo_root))?;
        if let Err(e) = worktree::create(Path::new(&repo_root), &target, branch, &default_branch) {
            tmux::show_error(&format!("Worktree creation failed:\n\n{e:#}"))?;
            return Ok(());
        }
    }
    let target_abs = worktree::abspath(&target);
    let command = tmux::shell_command();
    tmux::select_or_create_window(branch, &target_abs, &command)?;
    Ok(())
}

fn run_cleanup() -> Result<()> {
    let repo_root = repo::find_repo_root().context("Failed to find repository root")?;
    std::env::set_current_dir(&repo_root)?;
    let worktree_dir = tmux::resolve_worktree_dir();
    let default_branch = repo::resolve_default_branch(Path::new(&repo_root))?;
    let auto_fetch = tmux::get_opt("worktree-auto-fetch").unwrap_or_else(|| "true".to_string());
    if auto_fetch != "false" {
        let _ = git::run_in(
            Path::new(&repo_root),
            &["fetch", "origin", &default_branch, "--no-tags"],
        );
    }
    let worktrees = worktree::list_worktrees(Path::new(&repo_root), &worktree_dir)?;
    let mut items = Vec::new();
    let mut paths = Vec::new();
    for wt in &worktrees {
        let branch = worktree::branch(wt).unwrap_or_else(|_| "?".to_string());
        let merged = merge::is_merged(wt, &default_branch).unwrap_or(false);
        let (marker, item) = if merged {
            ("✓ merged", format!("✓ merged  | {branch}"))
        } else {
            ("✗ active", format!("✗ active  | {branch}"))
        };
        let _ = marker;
        items.push(item);
        paths.push(wt.clone());
    }
    if items.is_empty() {
        tmux::show_error(&format!("No worktrees found in {worktree_dir}/ — press Enter to dismiss"))?;
        return Ok(());
    }
    let picked = select::fzf_pick(&items, "Remove worktree> ", "Enter to remove selected worktree", false)?;
    let idx = match picked {
        select::Choice::Pick(i) => i,
        _ => return Ok(()),
    };
    let wt_dir = &paths[idx];
    let branch = items[idx]
        .split('|')
        .nth(1)
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if let Err(e) = worktree::remove(Path::new(&repo_root), wt_dir, &branch) {
        tmux::show_error(&format!("Worktree removal failed:\n\n{e:#}"))?;
        return Ok(());
    }
    worktree::delete_branch(Path::new(&repo_root), &branch);
    let _ = tmux::run(&["display-message", &format!("Removed worktree: {branch}")]);
    Ok(())
}

pub fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || "_-./:@%+,=".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_plain_path_unchanged() {
        assert_eq!(shell_quote("/home/user/repo"), "/home/user/repo");
    }

    #[test]
    fn shell_quote_wraps_spaces_in_single_quotes() {
        assert_eq!(shell_quote("/home/user/my repo"), "'/home/user/my repo'");
    }

    #[test]
    fn shell_quote_escapes_single_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

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
}
