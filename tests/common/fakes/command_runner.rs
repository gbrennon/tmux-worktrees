#![allow(dead_code)]
use std::cell::RefCell;
use std::collections::HashMap;
use std::io;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::Output;

use tmux_worktrees::infrastructure::command_runner::CommandRunner;

/// Pre-configured response; stored raw because [`Output`] is not `Clone`.
#[derive(Clone)]
enum Entry {
    Success {
        status: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    Error(io::ErrorKind),
}

impl Entry {
    fn into_output(self) -> io::Result<Output> {
        match self {
            Self::Success {
                status,
                stdout,
                stderr,
            } => Ok(Output {
                status: ExitStatusExt::from_raw(status),
                stdout,
                stderr,
            }),
            Self::Error(kind) => Err(io::Error::new(kind, "command not found")),
        }
    }
}

/// A fake [`CommandRunner`] keyed by `"program:arg1:arg2:…"`.
///
/// Unrecognised keys return a process-not-found error so tests fail fast.
#[derive(Clone)]
pub struct FakeRunner {
    entries: RefCell<HashMap<String, Entry>>,
}

impl FakeRunner {
    pub fn new() -> Self {
        Self {
            entries: RefCell::new(HashMap::new()),
        }
    }

    /// Register a success response.  `args` is colon-joined arguments
    /// (e.g. `"show-option:-gv:@myopt"`).
    pub fn ok(&self, program: &str, args: &str, status: i32, stdout: &str, stderr: &str) {
        let key = format!("{}:{}", program, args);
        self.entries.borrow_mut().insert(
            key,
            Entry::Success {
                status,
                stdout: stdout.as_bytes().to_vec(),
                stderr: stderr.as_bytes().to_vec(),
            },
        );
    }

    /// Register an error (command not found / failed to spawn).
    pub fn err(&self, program: &str, args: &str) {
        let key = format!("{}:{}", program, args);
        self.entries
            .borrow_mut()
            .insert(key, Entry::Error(io::ErrorKind::NotFound));
    }

    fn lookup(&self, program: &str, args: &[&str]) -> io::Result<Output> {
        let key = format!("{}:{}", program, args.join(":"));
        self.entries
            .borrow()
            .get(&key)
            .cloned()
            .unwrap_or(Entry::Error(io::ErrorKind::NotFound))
            .into_output()
    }
}

impl CommandRunner for FakeRunner {
    fn run(&self, program: &str, args: &[&str], _cwd: Option<&Path>) -> io::Result<Output> {
        self.lookup(program, args)
    }
}
