use std::path::Path;

pub struct MergeChecker;

impl MergeChecker {
    pub fn is_merged(
        &self,
        worktree_dir: &Path,
        original_branch: &str,
        check_merged: impl FnOnce(&Path, &str) -> Result<bool, String>,
    ) -> Result<bool, String> {
        let remote_ref = format!("origin/{original_branch}");
        check_merged(worktree_dir, &remote_ref)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    const MERGED_RESULT: Result<bool, String> = Ok(true);

    #[test]
    fn merge_checker_constructs_remote_ref() {
        let checker = MergeChecker;
        let worktree_dir = PathBuf::from("/tmp/test");
        let result = checker.is_merged(&worktree_dir, "main", |_, _| Ok(true));
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn merge_checker_with_develop_branch() {
        let checker = MergeChecker;
        let worktree_dir = PathBuf::from("/tmp/test");
        let captured_remote = RefCell::new(String::new());
        let result = checker.is_merged(&worktree_dir, "develop", |_, remote| {
            *captured_remote.borrow_mut() = remote.to_string();
            MERGED_RESULT
        });
        assert!(result.unwrap());
        assert_eq!(*captured_remote.borrow(), "origin/develop");
    }
}
