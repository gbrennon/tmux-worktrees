use std::path::Path;

pub struct ProjectLocator;

impl ProjectLocator {
    pub fn from_env(&self) -> Option<String> {
        find_project_root_from_env()
    }

    pub fn by_walking(&self, start: &Path) -> Option<String> {
        find_project_root_by_walking(start)
    }

    pub fn determine_default_branch(&self, global: &str, local: &str, current: &str) -> String {
        determine_default_branch(global, local, current)
    }

    pub fn ensure_workspace_directory(&self, project_root: &Path, workspace_dir: &str) -> anyhow::Result<()> {
        ensure_workspace_directory_exists(project_root, workspace_dir)
    }
}

fn find_project_root_from_env() -> Option<String> {
    if let Ok(root) = std::env::var("TMUX_WORKTREES_ROOT") {
        let root = root.trim().to_string();
        if !root.is_empty() && Path::new(&root).join(".git").is_dir() {
            return Some(root);
        }
    }
    None
}

fn find_project_root_by_walking(start: &Path) -> Option<String> {
    let mut dir = start.to_path_buf();
    loop {
        if dir.join(".git").is_dir() {
            return Some(dir.to_string_lossy().into_owned());
        }
        match dir.parent() {
            Some(parent) if parent.as_os_str() != dir.as_os_str() => {
                dir = parent.to_path_buf();
            }
            _ => break,
        }
    }
    if Path::new("/.git").is_dir() {
        return Some("/".to_string());
    }
    None
}

fn determine_default_branch(global: &str, local: &str, current: &str) -> String {
    if !global.is_empty() {
        return global.to_string();
    }
    if !local.is_empty() {
        return local.to_string();
    }
    if !current.is_empty() {
        return current.to_string();
    }
    "main".to_string()
}

fn ensure_workspace_directory_exists(project_root: &Path, workspace_dir: &str) -> anyhow::Result<()> {
    let dir = project_root.join(workspace_dir);
    if !dir.is_dir() {
        std::fs::create_dir(&dir)?;
        let exclude = project_root.join(".git").join("info").join("exclude");
        if exclude.exists() {
            let content = std::fs::read_to_string(&exclude).unwrap_or_default();
            let line = format!("{workspace_dir}/");
            if !content.lines().any(|l| l.trim_end() == line) {
                if let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&exclude) {
                    use std::io::Write;
                    let _ = f.write_all(format!("\n{line}\n").as_bytes());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn project_locator_from_env_returns_some_when_valid() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir(project.join(".git")).unwrap();
        std::env::set_var("TMUX_WORKTREES_ROOT", project.to_str().unwrap());
        let result = locator.from_env();
        assert_eq!(result, Some(project.to_string_lossy().into_owned()));
        std::env::remove_var("TMUX_WORKTREES_ROOT");
    }

    #[test]
    fn project_locator_from_env_returns_none_when_not_set() {
        let locator = ProjectLocator;
        std::env::remove_var("TMUX_WORKTREES_ROOT");
        let result = locator.from_env();
        assert_eq!(result, None);
    }

    #[test]
    fn project_locator_from_env_returns_none_when_not_git() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        std::env::set_var("TMUX_WORKTREES_ROOT", dir.path().to_str().unwrap());
        let result = locator.from_env();
        assert_eq!(result, None);
        std::env::remove_var("TMUX_WORKTREES_ROOT");
    }

    #[test]
    fn project_locator_by_walking_finds_git_dir() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        let sub = project.join("subdir");
        fs::create_dir_all(&sub).unwrap();
        fs::create_dir(project.join(".git")).unwrap();
        let result = locator.by_walking(&sub);
        assert_eq!(result, Some(project.to_string_lossy().into_owned()));
    }

    #[test]
    fn project_locator_by_walking_returns_none_when_no_git() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let result = locator.by_walking(dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn project_locator_determine_default_branch_priority() {
        let locator = ProjectLocator;
        assert_eq!(locator.determine_default_branch("global", "", ""), "global");
        assert_eq!(locator.determine_default_branch("", "local", ""), "local");
        assert_eq!(locator.determine_default_branch("", "", "current"), "current");
        assert_eq!(locator.determine_default_branch("", "", ""), "main");
    }

    #[test]
    fn project_locator_ensure_workspace_directory_creates_dir() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(project.join(".git").join("info")).unwrap();
        let workspace_dir = ".workspaces";
        let result = locator.ensure_workspace_directory(&project, workspace_dir);
        assert!(result.is_ok());
        assert!(project.join(workspace_dir).is_dir());
    }
}