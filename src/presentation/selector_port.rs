use crate::core::error::Result;
use crate::core::selector::{SelectionResult, Selector};

/// Port for running the interactive selector UI — injectable via
/// dependency inversion so the rest of the presentation layer
/// can be tested without a real terminal.
pub trait SelectorRunner {
    fn run_selector(
        &self,
        selector: &mut Selector,
        filtered: &[usize],
        prompt: &str,
        header: &str,
    ) -> Result<SelectionResult>;
}
