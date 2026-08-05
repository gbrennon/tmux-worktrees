use std::io;
use std::path::Path;
use std::process::{Command, Output};

/// Abstracts command execution so infrastructure adapters can be tested
/// without spawning real processes.  The system implementation delegates to
/// [`std::process::Command`]; fakes return pre-configured outputs.
pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str], cwd: Option<&Path>) -> io::Result<Output>;
}

/// The real command runner — delegates to [`std::process::Command`].
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemCommandRunner;

impl CommandRunner for SystemCommandRunner {
    fn run(&self, program: &str, args: &[&str], cwd: Option<&Path>) -> io::Result<Output> {
        let mut cmd = Command::new(program);
        cmd.args(args);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        cmd.output()
    }
}
