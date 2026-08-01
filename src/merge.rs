use anyhow::Result;
use std::path::Path;

use crate::git;

pub fn is_merged(wt_dir: &Path, default_branch: &str) -> Result<bool> {
    let remote = format!("origin/{default_branch}");
    let (status, _, _) = git::run_in(
        wt_dir,
        &["merge-base", "--is-ancestor", "HEAD", &remote],
    )?;
    Ok(status == 0)
}
