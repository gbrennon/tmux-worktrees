use std::fmt;
use std::str::FromStr;

/// User-facing CLI commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Choose,
    CreateWorktree,
    Cleanup,
}

/// Returned when a command string does not match any variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCommand(pub String);

impl fmt::Display for InvalidCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid command: {}", self.0)
    }
}

impl std::error::Error for InvalidCommand {}

impl FromStr for Command {
    type Err = InvalidCommand;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "choose" => Ok(Self::Choose),
            "create-worktree" => Ok(Self::CreateWorktree),
            "cleanup" => Ok(Self::Cleanup),
            other => Err(InvalidCommand(other.to_string())),
        }
    }
}

impl Command {
    /// Human-readable display string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Choose => "choose",
            Self::CreateWorktree => "create-worktree",
            Self::Cleanup => "cleanup",
        }
    }

    /// Whether this command requires an interactive terminal.
    pub fn is_interactive(self) -> bool {
        matches!(self, Self::Choose | Self::Cleanup)
    }
}
