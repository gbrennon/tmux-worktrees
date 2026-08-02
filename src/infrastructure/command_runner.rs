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

// ---------------------------------------------------------------------------
// Shared test-support — stores raw parts so the map can be cloned.
// ---------------------------------------------------------------------------
#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::os::unix::process::ExitStatusExt;

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
            let key = format!("{program}:{args}");
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
            let key = format!("{program}:{args}");
            self.entries
                .borrow_mut()
                .insert(key, Entry::Error(io::ErrorKind::NotFound));
        }

        fn lookup(&self, program: &str, args: &[&str]) -> io::Result<Output> {
            let key = format!("{program}:{}", args.join(":"));
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
}
