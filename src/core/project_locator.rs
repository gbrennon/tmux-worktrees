use std::path::Path;

pub struct ProjectLocator;

impl ProjectLocator {
    pub fn from_env() -> Option<String> {
        find_project_root_from_env()
    }

    pub fn by_walking(&self, start: &Path) -> Option<String> {
        find_project_root_by_walking(start)
    }

    pub fn determine_default_branch(&self, global: &str, local: &str, current: &str) -> String {
        determine_default_branch(global, local, current)
    }

    pub fn ensure_workspace_directory(
        &self,
        project_root: &Path,
        workspace_dir: &str,
    ) -> anyhow::Result<()> {
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

fn ensure_workspace_directory_exists(
    project_root: &Path,
    workspace_dir: &str,
) -> anyhow::Result<()> {
    let dir = project_root.join(workspace_dir);
    if !dir.is_dir() {
        std::fs::create_dir(&dir)?;
        let exclude = project_root.join(".git").join("info").join("exclude");
        if exclude.exists() {
            let content = std::fs::read_to_string(&exclude).unwrap_or_default();
            let line = format!("{workspace_dir}/");
            if !content.lines().any(|l| l.trim_end() == line)
                && let Ok(mut f) = std::fs::OpenOptions::new().append(true).open(&exclude)
            {
                use std::io::Write;
                let _ = f.write_all(format!("\n{line}\n").as_bytes());
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
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir(project.join(".git")).unwrap();
        unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", project.to_str().unwrap()) };
        let result = ProjectLocator::from_env();
        assert_eq!(result, Some(project.to_string_lossy().into_owned()));
        unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
    }

    #[test]
    fn project_locator_from_env_returns_none_when_not_set() {
        unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
        let result = ProjectLocator::from_env();
        assert_eq!(result, None);
    }

    #[test]
    fn project_locator_from_env_returns_none_when_not_git() {
        let dir = tempdir().unwrap();
        unsafe { std::env::set_var("TMUX_WORKTREES_ROOT", dir.path().to_str().unwrap()) };
        let result = ProjectLocator::from_env();
        assert_eq!(result, None);
        unsafe { std::env::remove_var("TMUX_WORKTREES_ROOT") };
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
        assert_eq!(
            locator.determine_default_branch("", "", "current"),
            "current"
        );
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
    #[test]
    fn project_locator_ensure_workspace_directory_already_exists() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        fs::create_dir_all(project.join(".git").join("info")).unwrap();
        let workspace_dir = ".workspaces";
        fs::create_dir(project.join(workspace_dir)).unwrap();
        let result = locator.ensure_workspace_directory(&project, workspace_dir);
        assert!(result.is_ok());
        assert!(project.join(workspace_dir).is_dir());
    }

    #[test]
    fn project_locator_ensure_workspace_directory_updates_exclude_when_needed() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        let info_dir = project.join(".git").join("info");
        fs::create_dir_all(&info_dir).unwrap();
        // Write exclude without the workspace dir line
        fs::write(info_dir.join("exclude"), "# existing exclude\n").unwrap();
        let workspace_dir = ".workspaces";
        let result = locator.ensure_workspace_directory(&project, workspace_dir);
        assert!(result.is_ok());
        assert!(project.join(workspace_dir).is_dir());
        let exclude_content = fs::read_to_string(info_dir.join("exclude")).unwrap();
        assert!(exclude_content.contains(".workspaces/"));
    }

    #[test]
    fn project_locator_ensure_workspace_directory_no_update_when_already_present() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        let info_dir = project.join(".git").join("info");
        fs::create_dir_all(&info_dir).unwrap();
        // Write exclude WITH the workspace dir line already present
        let original = "# existing exclude\n.workspaces/\n";
        fs::write(info_dir.join("exclude"), original).unwrap();
        let workspace_dir = ".workspaces";
        let result = locator.ensure_workspace_directory(&project, workspace_dir);
        assert!(result.is_ok());
        assert!(project.join(workspace_dir).is_dir());
        let exclude_content = fs::read_to_string(info_dir.join("exclude")).unwrap();
        // Content should be unchanged (no duplicate line appended)
        assert_eq!(exclude_content, original);
    }

    #[test]
    fn project_locator_ensure_workspace_directory_errors_when_create_dir_fails() {
        let locator = ProjectLocator;
        let dir = tempdir().unwrap();
        let project = dir.path().join("project");
        let info_dir = project.join(".git").join("info");
        fs::create_dir_all(&info_dir).unwrap();
        // Create a FILE at the workspace dir path, so create_dir fails
        let workspace_dir = ".workspaces";
        fs::write(project.join(workspace_dir), "blocker").unwrap();
        let result = locator.ensure_workspace_directory(&project, workspace_dir);
        assert!(result.is_err());
    }
}
