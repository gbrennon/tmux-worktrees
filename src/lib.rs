pub mod core;
pub mod infrastructure;
pub mod presentation;
pub mod utils;

use infrastructure::container::Container;
use presentation::cli::Cli;

/// Composition root: builds a fully wired [`Cli`] backed by production
/// adapters (real tmux and git executors).
pub fn build_cli() -> Cli {
    Container::build_cli()
}
