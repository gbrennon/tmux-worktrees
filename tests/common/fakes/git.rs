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
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["branch", "-D", _] => Ok((0, String::new(), String::new())),
            _ => unimplemented!(),
        }
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
    pub rev_parse_outputs: Vec<String>,
    pub rev_parse_index: RefCell<usize>,
    pub remove_calls: std::rc::Rc<RefCell<Vec<String>>>,
    pub deleted_branches: std::rc::Rc<RefCell<Vec<String>>>,
    pub remove_outcomes: Vec<(i32, String)>,
    pub remove_index: RefCell<usize>,
    pub branch_delete_outcomes: Vec<(i32, String)>,
    pub branch_delete_index: RefCell<usize>,
}

impl FakeGitForCleanup {
    pub fn new(porcelain: &str) -> Self {
        Self {
            porcelain: porcelain.to_string(),
            rev_parse_outputs: vec!["feat-a".into()],
            rev_parse_index: RefCell::new(0),
            remove_calls: std::rc::Rc::new(RefCell::new(Vec::new())),
            deleted_branches: std::rc::Rc::new(RefCell::new(Vec::new())),
            remove_outcomes: vec![(0, String::new())],
            remove_index: RefCell::new(0),
            branch_delete_outcomes: vec![(0, String::new())],
            branch_delete_index: RefCell::new(0),
        }
    }

    pub fn with_branches(mut self, branches: &[&str]) -> Self {
        self.rev_parse_outputs = branches.iter().map(|branch| (*branch).into()).collect();
        self
    }

    pub fn with_remove_failure(mut self) -> Self {
        self.remove_outcomes = vec![(1, "worktree locked".into())];
        self.remove_index = RefCell::new(0);
        self
    }

    pub fn with_remove_outcomes(mut self, outcomes: &[(i32, &str)]) -> Self {
        self.remove_outcomes = outcomes
            .iter()
            .map(|(status, stderr)| (*status, (*stderr).into()))
            .collect();
        self.remove_index = RefCell::new(0);
        self
    }
    pub fn with_branch_delete_outcomes(mut self, outcomes: &[(i32, &str)]) -> Self {
        self.branch_delete_outcomes = outcomes
            .iter()
            .map(|(status, stderr)| (*status, (*stderr).into()))
            .collect();
        self.branch_delete_index = RefCell::new(0);
        self
    }
}
impl GitPort for FakeGitForCleanup {
    fn run_in(&self, d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["branch", "-D", branch] => {
                let mut index = self.branch_delete_index.borrow_mut();
                let (status, stderr) = self
                    .branch_delete_outcomes
                    .get(*index)
                    .or_else(|| self.branch_delete_outcomes.last())
                    .cloned()
                    .unwrap_or((0, String::new()));
                *index += 1;
                drop(index);
                if status == 0 {
                    self.deleted_branches
                        .borrow_mut()
                        .push((*branch).to_string());
                }
                Ok((status, String::new(), stderr))
            }
            ["worktree", "list", "--porcelain"] => {
                let porcelain = self.porcelain.replace("__ROOT__", &d.to_string_lossy());
                Ok((0, porcelain, String::new()))
            }
            ["rev-parse", "--abbrev-ref", "HEAD"] => {
                let mut index = self.rev_parse_index.borrow_mut();
                let branch = self
                    .rev_parse_outputs
                    .get(*index)
                    .or_else(|| self.rev_parse_outputs.last())
                    .cloned()
                    .unwrap_or_default();
                *index += 1;
                Ok((0, branch, String::new()))
            }
            ["merge-base", "--is-ancestor", "HEAD", _] => Ok((0, String::new(), String::new())),
            ["fetch", "origin", _, "--no-tags"] => Ok((0, String::new(), String::new())),
            ["worktree", "remove", "--force", path] => {
                self.remove_calls.borrow_mut().push((*path).to_string());
                let mut index = self.remove_index.borrow_mut();
                let (status, stderr) = self
                    .remove_outcomes
                    .get(*index)
                    .or_else(|| self.remove_outcomes.last())
                    .cloned()
                    .unwrap_or((0, String::new()));
                *index += 1;
                Ok((status, String::new(), stderr))
            }
            _ => unimplemented!("unexpected git run_in: {:?}", args),
        }
    }
    fn silent_in(&self, _d: &Path, args: &[&str]) -> Result<()> {
        match args {
            ["branch", "-D", branch] => {
                self.deleted_branches
                    .borrow_mut()
                    .push((*branch).to_string());
                Ok(())
            }
            _ => unimplemented!("unexpected git silent_in: {:?}", args),
        }
    }
}

pub struct FakeGitForChoose {
    repo_path: String,
    porcelain: String,
    branch_name: String,
    reject_worktree_add: bool,
}
impl FakeGitForChoose {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
            porcelain: format!(
                "worktree {}\nworktree {}/.worktrees/feat-a\n",
                repo_path, repo_path
            ),
            branch_name: String::new(),
            reject_worktree_add: false,
        }
    }

    pub fn existing_worktree(repo_path: &str, worktree_path: &str, branch_name: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
            porcelain: format!("worktree {}\nworktree {}\n", repo_path, worktree_path),
            branch_name: branch_name.to_string(),
            reject_worktree_add: true,
        }
    }
}
impl GitPort for FakeGitForChoose {
    fn run_in(&self, _d: &Path, args: &[&str]) -> Result<(i32, String, String)> {
        match args {
            ["worktree", "list", "--porcelain"] => Ok((0, self.porcelain.clone(), String::new())),
            ["rev-parse", "--abbrev-ref", "HEAD"] => {
                Ok((0, self.branch_name.clone(), String::new()))
            }
            ["config", "--global", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["config", "init.defaultBranch"] => Ok((0, "main".into(), String::new())),
            ["branch", "--show-current"] => Ok((0, "main".into(), String::new())),
            ["show-ref", "--verify", "--quiet", _] => Ok((1, String::new(), String::new())),
            ["worktree", "add", _, "-b", _, _] if self.reject_worktree_add => {
                panic!("unexpected git worktree add")
            }
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
            ["branch", "-D", _] => Ok((0, String::new(), String::new())),
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
