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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_choose() {
        assert_eq!("choose".parse::<Command>().unwrap(), Command::Choose);
    }

    #[test]
    fn parse_create_worktree() {
        assert_eq!(
            "create-worktree".parse::<Command>().unwrap(),
            Command::CreateWorktree
        );
    }

    #[test]
    fn parse_cleanup() {
        assert_eq!("cleanup".parse::<Command>().unwrap(), Command::Cleanup);
    }

    #[test]
    fn parse_unknown() {
        assert!("bogus".parse::<Command>().is_err());
    }

    #[test]
    fn interactive_commands() {
        assert!(Command::Choose.is_interactive());
        assert!(Command::Cleanup.is_interactive());
        assert!(!Command::CreateWorktree.is_interactive());
    }

    #[test]
    fn as_str_returns_correct_variants() {
        assert_eq!(Command::Choose.as_str(), "choose");
        assert_eq!(Command::CreateWorktree.as_str(), "create-worktree");
        assert_eq!(Command::Cleanup.as_str(), "cleanup");
    }

    #[test]
    fn invalid_command_display_shows_message() {
        assert_eq!(
            InvalidCommand("bogus-command".into()).to_string(),
            "invalid command: bogus-command"
        );
    }

    #[test]
    fn invalid_command_is_std_error() {
        let err = InvalidCommand("test".into());
        let _: &dyn std::error::Error = &err;
    }

}
