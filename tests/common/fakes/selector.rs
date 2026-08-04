use std::cell::RefCell;

use tmux_worktrees::core::error::Result;
use tmux_worktrees::core::selector::{SelectionResult, Selector};
use tmux_worktrees::presentation::selector_port::SelectorRunner;

/// Selector runner for tests that returns a canned result.
pub struct StubSelector;
impl SelectorRunner for StubSelector {
    fn run_selector(
        &self,
        _selector: &mut Selector,
        _filtered: &[usize],
        _prompt: &str,
        _header: &str,
    ) -> Result<SelectionResult> {
        unimplemented!()
    }
}

// ===========================================================================
// FakeSelector — returns configurable SelectionResult values from a queue
// ===========================================================================

pub struct FakeSelector {
    pub results: RefCell<Vec<SelectionResult>>,
}
impl FakeSelector {
    pub fn new(results: Vec<SelectionResult>) -> Self {
        Self {
            results: RefCell::new(results),
        }
    }
}
impl SelectorRunner for FakeSelector {
    fn run_selector(
        &self,
        _selector: &mut Selector,
        _filtered: &[usize],
        _prompt: &str,
        _header: &str,
    ) -> Result<SelectionResult> {
        let mut results = self.results.borrow_mut();
        if !results.is_empty() {
            Ok(results.remove(0))
        } else {
            Ok(SelectionResult::Cancelled)
        }
    }
}
