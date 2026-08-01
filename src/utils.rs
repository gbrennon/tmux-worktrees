pub fn shell_quote(s: &str) -> String {
    if s.chars().all(|c| c.is_alphanumeric() || "_-./:@%+,=".contains(c)) {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_plain_path_unchanged() {
        assert_eq!(shell_quote("/home/user/repo"), "/home/user/repo");
    }

    #[test]
    fn shell_quote_wraps_spaces_in_single_quotes() {
        assert_eq!(shell_quote("/home/user/my repo"), "'/home/user/my repo'");
    }

    #[test]
    fn shell_quote_escapes_single_quote() {
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }
}