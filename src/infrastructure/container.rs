use crate::infrastructure::git_executor::GitExecutor;
use crate::infrastructure::tmux_executor::TmuxExecutor;
use crate::presentation::cli::Cli;

/// Dependency-injection container that wires production adapters into the
/// presentation layer.
pub struct Container;

impl Container {
    /// Build a fully wired [`Cli`] backed by real tmux and git executors.
    pub fn build_cli() -> Cli {
        Cli::new(
            Box::new(TmuxExecutor::default()),
            Box::new(GitExecutor::default()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_builds_cli_without_panicking() {
        let _cli = Container::build_cli();
    }
}
