#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    CtrlC,
    CtrlU,
    Backspace,
    Down,
    Up,
    Esc,
    Enter,
    Unknown,
}

pub struct Selector {
    items: Vec<String>,
    query: String,
    selected_index: usize,
    allow_custom: bool,
}

impl Selector {
    pub fn new(items: Vec<String>, allow_custom: bool) -> Self {
        Self {
            items,
            query: String::new(),
            selected_index: 0,
            allow_custom,
        }
    }

    pub fn filter(&self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            return (0..self.items.len()).collect();
        }

        let q = query.to_lowercase();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, s)| s.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    pub fn clamp_selection(&mut self, filtered_len: usize) {
        if filtered_len == 0 {
            self.selected_index = 0;
        } else if self.selected_index >= filtered_len {
            self.selected_index = filtered_len - 1;
        }
    }

    pub fn process_key(&mut self, key: Key, filtered: &[usize]) -> Option<SelectionResult> {
        match key {
            Key::CtrlC => Some(SelectionResult::Cancelled),
            Key::CtrlU => {
                self.query.clear();
                self.selected_index = 0;
                None
            }
            Key::Char(c) => {
                self.query.push(c);
                self.selected_index = 0;
                None
            }
            Key::Backspace => {
                self.query.pop();
                self.selected_index = 0;
                None
            }
            Key::Down => {
                let candidate = self.selected_index.saturating_add(1);
                self.selected_index = if filtered.is_empty() {
                    0
                } else if candidate >= filtered.len() {
                    filtered.len() - 1
                } else {
                    candidate
                };
                None
            }
            Key::Up => {
                self.selected_index = self.selected_index.saturating_sub(1);
                None
            }
            Key::Esc => Some(SelectionResult::Cancelled),
            Key::Enter => {
                if let Some(&idx) = filtered.get(self.selected_index) {
                    Some(SelectionResult::Selected(idx))
                } else if self.allow_custom && !self.query.trim().is_empty() {
                    Some(SelectionResult::Custom(self.query.trim().to_string()))
                } else {
                    None
                }
            }
            Key::Unknown => None,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn items(&self) -> &[String] {
        &self.items
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionResult {
    Selected(usize),
    Custom(String),
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_new_initializes() {
        let items = vec!["a".to_string(), "b".to_string()];
        let selector = Selector::new(items.clone(), true);
        assert_eq!(selector.items(), &items);
        assert_eq!(selector.query(), "");
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn filter_empty_query_returns_all() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = Selector::new(items.clone(), false).filter("");
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn filter_matches_substring() {
        let items = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let result = Selector::new(items.clone(), false).filter("ba");
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn filter_case_insensitive() {
        let items = vec!["FooBar".to_string(), "baz".to_string()];
        let result = Selector::new(items.clone(), false).filter("foobar");
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn filter_no_match_returns_empty() {
        let items = vec!["foo".to_string(), "bar".to_string()];
        let result = Selector::new(items.clone(), false).filter("xyz");
        assert_eq!(result, Vec::<usize>::new());
    }

    #[test]
    fn clamp_selection_empty_filtered() {
        let items = vec!["a".to_string()];
        let mut selector = Selector::new(items, false);
        selector.clamp_selection(0);
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn clamp_selection_within_bounds() {
        let items = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let mut selector = Selector::new(items, false);
        selector.clamp_selection(5);
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn clamp_selection_exceeds_bounds() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, false);
        for _ in 0..10 {
            selector.process_key(Key::Down, &[0, 1, 2]);
        }
        assert_eq!(selector.selected_index(), 2);
        selector.clamp_selection(2);
        assert_eq!(selector.selected_index(), 1);
    }

    #[test]
    fn process_key_ctrl_c_returns_cancelled() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::CtrlC, &[0, 1, 2]);
        assert_eq!(result, Some(SelectionResult::Cancelled));
        assert_eq!(selector.query(), "");
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn process_key_ctrl_u_clears_query() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        for c in "test".chars() {
            selector.process_key(Key::Char(c), &[0, 1, 2]);
        }
        assert_eq!(selector.query(), "test");
        let result = selector.process_key(Key::CtrlU, &[0, 1, 2]);
        assert_eq!(result, None);
        assert_eq!(selector.query(), "");
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn process_key_char_appends_to_query() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Char('a'), &[0, 1]);
        assert_eq!(result, None);
        assert_eq!(selector.query(), "a");
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn process_key_backspace_removes_last_char() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        for c in "test".chars() {
            selector.process_key(Key::Char(c), &[0, 1, 2]);
        }
        selector.process_key(Key::Down, &[0, 1, 2]);
        selector.process_key(Key::Down, &[0, 1, 2]);
        assert_eq!(selector.query(), "test");
        assert_eq!(selector.selected_index(), 2);
        let result = selector.process_key(Key::Backspace, &[0, 1, 2]);
        assert_eq!(result, None);
        assert_eq!(selector.query(), "tes");
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn process_key_down_increments_selection() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Down, &[0, 1, 2]);
        assert_eq!(result, None);
        assert_eq!(selector.selected_index(), 1);
    }

    #[test]
    fn process_key_up_decrements_selection() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        selector.process_key(Key::Down, &[0, 1, 2]);
        selector.process_key(Key::Down, &[0, 1, 2]);
        assert_eq!(selector.selected_index(), 2);
        let result = selector.process_key(Key::Up, &[0, 1, 2]);
        assert_eq!(result, None);
        assert_eq!(selector.selected_index(), 1);
    }

    #[test]
    fn process_key_up_at_zero_stays_zero() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Up, &[0, 1, 2]);
        assert_eq!(result, None);
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn process_key_esc_returns_cancelled() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Esc, &[0, 1]);
        assert_eq!(result, Some(SelectionResult::Cancelled));
    }

    #[test]
    fn process_key_enter_selects_item() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        selector.process_key(Key::Down, &[0, 1, 2]);
        assert_eq!(selector.selected_index(), 1);
        let result = selector.process_key(Key::Enter, &[0, 1, 2]);
        assert_eq!(result, Some(SelectionResult::Selected(1)));
    }

    #[test]
    fn process_key_enter_creates_custom_when_allowed_and_query() {
        let items = vec!["a".to_string()];
        let mut selector = Selector::new(items, true);
        for c in "new-branch".chars() {
            selector.process_key(Key::Char(c), &[]);
        }
        assert_eq!(selector.query(), "new-branch");
        let result = selector.process_key(Key::Enter, &[]);
        assert_eq!(
            result,
            Some(SelectionResult::Custom("new-branch".to_string()))
        );
    }

    #[test]
    fn process_key_enter_no_custom_when_not_allowed() {
        let items = Vec::<String>::new();
        let mut selector = Selector::new(items, false);
        for c in "new-branch".chars() {
            selector.process_key(Key::Char(c), &[]);
        }
        assert_eq!(selector.query(), "new-branch");
        let result = selector.process_key(Key::Enter, &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_enter_no_custom_when_query_empty() {
        let items = Vec::<String>::new();
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Enter, &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn selector_filter_inline_works() {
        let items = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let selector = Selector::new(items.clone(), true);
        let result = selector.filter("ba");
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn selector_clamp_inline_works() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        selector.clamp_selection(3);
        assert_eq!(selector.selected_index(), 0);
        selector.clamp_selection(0);
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn selector_process_key_inline_works() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::CtrlC, &[0, 1]);
        assert_eq!(result, Some(SelectionResult::Cancelled));
        assert_eq!(selector.query(), "");
    }

    #[test]
    fn process_key_unknown_key_returns_none() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key(Key::Unknown, &[0, 1]);
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_enter_with_oob_index_returns_none() {
        let items = vec!["x".to_string()];
        let mut selector = Selector::new(items, false);
        for _ in 0..5 {
            selector.process_key(Key::Down, &[]);
        }
        let result = selector.process_key(Key::Enter, &[]);
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_enter_empty_query_no_custom_when_not_allowed() {
        let items = Vec::<String>::new();
        let mut selector = Selector::new(items, false);
        let result = selector.process_key(Key::Enter, &[]);
        assert_eq!(result, None);
    }
}
