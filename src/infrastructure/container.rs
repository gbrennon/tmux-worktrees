use crate::infrastructure::git_executor::GitExecutor;
use crate::infrastructure::tmux_executor::TmuxExecutor;
use crate::presentation::cli::Cli;
use crate::presentation::ratatui_loading::RatatuiLoading;
use crate::presentation::ratatui_selector::RatatuiSelector;

/// Dependency-injection container that wires production adapters into the
/// presentation layer.
pub struct Container;

impl Container {
    /// Build a fully wired [`Cli`] backed by real tmux and git executors.
    pub fn build_cli() -> Cli {
        Cli::new(
            Box::new(TmuxExecutor::default()),
            Box::new(GitExecutor::default()),
            Box::new(RatatuiSelector),
            Box::new(RatatuiLoading),
        )
    }
}
