/// Utility for shell-safe quoting of paths and arguments.
pub struct ShellQuoter;

impl ShellQuoter {
    /// Quote a string for safe use in a shell command.
    /// Plain alphanumeric + safe-symbol strings pass through unchanged;
    /// everything else is single-quoted with embedded quotes escaped.
    pub fn quote(s: &str) -> String {
        if s.chars()
            .all(|c| c.is_alphanumeric() || "_-./:@%+,=".contains(c))
        {
            s.to_string()
        } else {
            format!("'{}'", s.replace('\'', "'\\''"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_plain_path_unchanged() {
        assert_eq!(ShellQuoter::quote("/home/user/repo"), "/home/user/repo");
    }

    #[test]
    fn shell_quote_path_with_spaces_gets_quoted() {
        assert_eq!(
            ShellQuoter::quote("/home/user/my repo"),
            "'/home/user/my repo'"
        );
    }

    #[test]
    fn shell_quote_string_with_single_quote_escapes_correctly() {
        assert_eq!(ShellQuoter::quote("it's"), "'it'\\''s'");
    }
}
