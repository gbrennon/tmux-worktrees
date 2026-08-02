use std::path::{Path, PathBuf};

pub struct WorkspaceResolver;

impl WorkspaceResolver {
    pub fn format_directory_name(&self, branch: &str) -> String {
        branch.replace('/', "-")
    }

    pub fn resolve_path(&self, project_root: &Path, workspace_dir: &str, branch: &str) -> PathBuf {
        project_root
            .join(workspace_dir)
            .join(self.format_directory_name(branch))
    }

    pub fn extract_branch_from_display(&self, display: &str) -> Option<String> {
        extract_branch_name_from_display(display)
    }

    pub fn resolve_absolute(&self, target: &Path) -> String {
        resolve_absolute_path(target)
    }
}

fn extract_branch_name_from_display(display: &str) -> Option<String> {
    if let Some((_, branch)) = display.split_once("|") {
        Some(branch.trim().to_string())
    } else {
        None
    }
}

fn resolve_absolute_path(target: &Path) -> String {
    std::fs::canonicalize(target)
        .unwrap_or_else(|_| target.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_resolver_formats_directory_name() {
        let resolver = WorkspaceResolver;
        assert_eq!(resolver.format_directory_name("feature/foo"), "feature-foo");
        assert_eq!(resolver.format_directory_name("bug/fix/bar"), "bug-fix-bar");
    }

    #[test]
    fn workspace_resolver_resolves_path() {
        let resolver = WorkspaceResolver;
        let project_root = PathBuf::from("/home/user/project");
        let workspace_dir = ".workspaces";
        let branch = "feature/foo";
        let path = resolver.resolve_path(&project_root, workspace_dir, branch);
        assert_eq!(
            path,
            PathBuf::from("/home/user/project/.workspaces/feature-foo")
        );
    }

    #[test]
    fn workspace_resolver_extracts_branch_from_display() {
        let resolver = WorkspaceResolver;
        let display = "✓ merged  | feature/foo";
        let result = resolver.extract_branch_from_display(display);
        assert_eq!(result, Some("feature/foo".to_string()));
    }

    #[test]
    fn workspace_resolver_resolves_absolute_path() {
        let resolver = WorkspaceResolver;
        let path = PathBuf::from("/home/user/project");
        let result = resolver.resolve_absolute(&path);
        assert_eq!(result, "/home/user/project");
    }
    #[test]
    fn extract_branch_name_from_display_returns_none_when_no_separator() {
        // Direct function call
        let result = extract_branch_name_from_display("main");
        assert_eq!(result, None);
        // Via struct method
        let resolver = WorkspaceResolver;
        let result = resolver.extract_branch_from_display("main");
        assert_eq!(result, None);
    }
}
