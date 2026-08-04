// Integration tests for the Command enum and InvalidCommand error.

use tmux_worktrees::presentation::command::{Command, InvalidCommand};

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
    let err: Box<dyn std::error::Error> = Box::new(InvalidCommand("test".into()));
    // Just verify it can be downcast — source() is None
    assert!(err.source().is_none());
}
