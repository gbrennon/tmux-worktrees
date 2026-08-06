#![allow(dead_code)]
use std::cell::RefCell;
use std::path::Path;

use tmux_worktrees::core::error::Result;
use tmux_worktrees::core::ports::GitPort;

pub struct StubGit;
impl GitPort for StubGit {
    fn run_in(&self, _d: &Path, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitWorktreeAdd;
impl GitPort for FakeGitWorktreeAdd {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((0, String::new(), String::new()))
        } else {
            Ok((0, String::new(), String::new()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitWorktreeFail;
impl GitPort for FakeGitWorktreeFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((1, String::new(), "ref missing".into()))
        } else {
            Ok((1, String::new(), "fail".into()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitWorktreeAddLocal;
impl GitPort for FakeGitWorktreeAddLocal {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args[0] == "show-ref" {
            Ok((1, String::new(), String::new()))
        } else {
            Ok((0, String::new(), String::new()))
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitWorktreeRemove;
impl GitPort for FakeGitWorktreeRemove {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "remove", "--force", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitWorktreeRemoveFail;
impl GitPort for FakeGitWorktreeRemoveFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "remove", "--force", _] => Ok((1, String::new(), "remove failed".into())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitRevParseFail;
impl GitPort for FakeGitRevParseFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((1, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitRevParseOk;
impl GitPort for FakeGitRevParseOk {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((0, "feat/x\n".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitRevParseDetached;
impl GitPort for FakeGitRevParseDetached {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["rev-parse", "--abbrev-ref", "HEAD"] => Ok((0, "HEAD\n".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitWorktreeListFail;
impl GitPort for FakeGitWorktreeListFail {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((1, String::new(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitWorktreeListOk;
impl GitPort for FakeGitWorktreeListOk {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => {
                Ok((0, "worktree /tmp/repo\nworktree /tmp/repo/worktrees/feat-a\nworktree /tmp/repo/worktrees/fix-b\n".into(), String::new()))
            }
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitDefaultBranch;
impl GitPort for FakeGitDefaultBranch {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitBranchLocalOnly;
impl GitPort for FakeGitBranchLocalOnly {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "develop".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "develop".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitBranchFallbackCurrent;
impl GitPort for FakeGitBranchFallbackCurrent {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["config", "init.defaultBranch"] => Ok((1, String::new(), String::new())),
            ["branch", "--show-current"] => Ok((0, "trunk".into(), String::new())),
            _ => unimplemented!(),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        unimplemented!()
    }
}

pub struct FakeGitBranchDelete;
impl GitPort for FakeGitBranchDelete {
    fn run_in(&self, _d: &Path, _a: &[&str]) -> Result<(i32, String, String)> {
        unimplemented!()
    }
    fn silent_in(&self, _d: &Path, _args: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitCreate;
impl GitPort for FakeGitCreate {
    fn run_in(&self, _dir: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        if args.contains(&"init.defaultBranch") {
            Ok((0, "main".into(), String::new()))
        } else if args.contains(&"--show-current") {
            Ok((0, "main".into(), String::new()))
        } else if args.contains(&"show-ref") {
            Ok((1, String::new(), String::new()))
        } else if args.contains(&"worktree") && args.contains(&"add") {
            Ok((0, String::new(), String::new()))
        } else {
            unimplemented!("unexpected run_in args: {args:?}")
        }
    }
    fn silent_in(&self, _dir: &Path, _args: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitPorcelain {
    pub output: String,
}
impl FakeGitPorcelain {
    pub fn new(output: &str) -> Self {
        Self {
            output: output.to_string(),
        }
    }
}
impl GitPort for FakeGitPorcelain {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((0, self.output.clone(), String::new())),
            _ => unimplemented!("unexpected: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitForCleanup {
    pub porcelain: String,
    pub rev_parse_output: String,
    pub remove_status: i32,
    pub remove_stderr: String,
}
impl FakeGitForCleanup {
    pub fn new(porcelain: &str) -> Self {
        Self {
            porcelain: porcelain.to_string(),
            rev_parse_output: "feat-a".into(),
            remove_status: 0,
            remove_stderr: String::new(),
        }
    }
    pub fn with_remove_failure(mut self) -> Self {
        self.remove_status = 1;
        self.remove_stderr = "worktree locked".into();
        self
    }
}
impl GitPort for FakeGitForCleanup {
    fn run_in(&self, d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["worktree", "list", "--porcelain"] => {
                let porcelain = self.porcelain.replace("__ROOT__", &d.to_string_lossy());
                Ok((0, porcelain, String::new()))
            }
            ["rev-parse", "--abbrev-ref", "HEAD"] => {
                Ok((0, self.rev_parse_output.clone(), String::new()))
            }
            ["merge-base", "--is-ancestor", "HEAD", _] => Ok((0, String::new(), String::new())),
            ["fetch", "origin", _, "--no-tags"] => Ok((0, String::new(), String::new())),
            ["worktree", "remove", "--force", _] => Ok((
                self.remove_status,
                String::new(),
                self.remove_stderr.clone(),
            )),
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, args: &[&str]) -> Result<()> {
        match args {
            ["branch", "-D", _] => Ok(()),
            _ => unimplemented!("unexpected git silent_in: {:?}", args),
        }
    }
}

pub struct FakeGitForChoose {
    repo_path: String,
}
impl FakeGitForChoose {
    /// # Panics
    ///
    /// Never, as long as  is a valid &str.
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
        }
    }
}
impl GitPort for FakeGitForChoose {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((
                0,
                format!(
                    "worktree {}\nworktree {}/.worktrees/feat-a\n",
                    self.repo_path, self.repo_path
                ),
                String::new(),
            )),
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["show-ref", "--verify", "--quiet", _] => Ok((1, String::new(), String::new())),
            ["worktree", "add", _, "-b", _, _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, _a: &[&str]) -> Result<()> {
        Ok(())
    }
}

pub struct FakeGitLifecycle {
    pub porcelain: RefCell<String>,
    pub branch_name: RefCell<String>,
}

impl FakeGitLifecycle {
    pub fn new() -> Self {
        Self {
            porcelain: RefCell::new(String::new()),
            branch_name: RefCell::new("feat/lifecycle-test".into()),
        }
    }

    pub fn set_porcelain(&self, output: &str) {
        *self.porcelain.borrow_mut() = output.to_string();
    }

    pub fn set_branch(&self, name: &str) {
        *self.branch_name.borrow_mut() = name.to_string();
    }
}

impl GitPort for FakeGitLifecycle {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["show-ref", "--verify", "--quiet", _] => Ok((1, String::new(), String::new())),
            ["worktree", "add", _, "-b", branch, _] => {
                *self.branch_name.borrow_mut() = branch.to_string();
                Ok((0, String::new(), String::new()))
            }
            ["worktree", "list", "--porcelain"] => {
                Ok((0, self.porcelain.borrow().clone(), String::new()))
            }
            ["rev-parse", "--abbrev-ref", "HEAD"] => {
                Ok((0, self.branch_name.borrow().clone(), String::new()))
            }
            ["worktree", "remove", "--force", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }

    fn silent_in(&self, _d: &Path, args: &[&str]) -> Result<()> {
        match args {
            ["fetch", "origin", "--quiet"] => Ok(()),
            ["branch", "-D", _] => Ok(()),
            _ => unimplemented!("unexpected git silent_in: {:?}", args),
        }
    }
}
