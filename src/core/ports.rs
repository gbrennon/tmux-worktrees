use crate::core::error::Result;
use std::path::Path;

/// Port for tmux operations — defines the interface the application layer
/// depends on so infrastructure can be swapped for tests.
pub trait TmuxPort {
    fn show_error(&self, msg: &str) -> Result<()>;
    fn display_popup(&self, width: &str, height: &str, dir: &str, cmd: &str) -> Result<()>;
    fn resolve_workspace_dir(&self) -> String;
    fn resolve_shell_command(&self) -> String;
    fn select_or_create_window(&self, name: &str, path: &str, command: &str) -> Result<()>;
    fn get_option(&self, key: &str) -> Option<String>;
    fn show_environment(&self, var: &str) -> Result<Option<String>>;
    fn current_pane_path(&self) -> Result<String>;
    fn run(&self, args: &[&str]) -> Result<(i32, String, String)>;
    fn kill_window(&self, name: &str) -> Result<()>;
}

/// Port for git operations — defines the interface the application layer
/// depends on so infrastructure can be swapped for tests.
pub trait GitPort {
    fn run_in(&self, dir: &Path, args: &[&str]) -> Result<(i32, String, String)>;
    fn silent_in(&self, dir: &Path, args: &[&str]) -> Result<()>;
}
