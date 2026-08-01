use std::path::Path;

pub struct MergeChecker;

impl MergeChecker {
    pub fn is_merged_into_default(
        &self,
        workspace_dir: &Path,
        default_branch: &str,
        check_merged: impl FnOnce(&Path, &str) -> anyhow::Result<i32>
    ) -> anyhow::Result<bool> {
        let remote_ref = format!("origin/{default_branch}");
        let status = check_merged(workspace_dir, &remote_ref)?;
        Ok(status == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn merge_checker_constructs_remote_ref() {
        let checker = MergeChecker;
        let workspace_dir = PathBuf::from("/tmp/test");
        let result = checker.is_merged_into_default(&workspace_dir, "main", |_dir, remote| {
            assert_eq!(remote, "origin/main");
            Ok(0)
        });
        assert!(result.unwrap());
    }

    #[test]
    fn merge_checker_with_develop_branch() {
        let checker = MergeChecker;
        let workspace_dir = PathBuf::from("/tmp/test");
        let result = checker.is_merged_into_default(&workspace_dir, "develop", |_dir, remote| {
            assert_eq!(remote, "origin/develop");
            Ok(0)
        });
        assert!(result.unwrap());
    }

    #[test]
    fn merge_checker_returns_true_when_status_zero() {
        let checker = MergeChecker;
        let workspace_dir = PathBuf::from("/tmp/test");
        let result = checker.is_merged_into_default(&workspace_dir, "main", |_, _| Ok(0));
        assert!(result.unwrap());
    }

    #[test]
    fn merge_checker_returns_false_when_status_non_zero() {
        let checker = MergeChecker;
        let workspace_dir = PathBuf::from("/tmp/test");
        let result = checker.is_merged_into_default(&workspace_dir, "main", |_, _| Ok(1));
        assert!(!result.unwrap());
    }
}
