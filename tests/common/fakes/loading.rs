#![allow(dead_code)]
use std::cell::RefCell;
use std::rc::Rc;

use tmux_worktrees::core::error::Result;
use tmux_worktrees::presentation::loading_port::LoadingRunner;

pub struct FakeLoading {
    pub labels: Rc<RefCell<Vec<String>>>,
}
impl FakeLoading {
    pub fn new() -> Self {
        Self {
            labels: Rc::new(RefCell::new(Vec::new())),
        }
    }
}
impl LoadingRunner for FakeLoading {
    fn run_loading(&self, label: &str, job: Box<dyn FnOnce() -> Result<()> + '_>) -> Result<()> {
        self.labels.borrow_mut().push(label.to_string());
        job()
    }
}
