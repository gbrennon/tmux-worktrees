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
        filter_items_by_query(&self.items, query)
    }

    pub fn clamp_selection(&mut self, filtered_len: usize) {
        self.selected_index = clamp_selection_index(self.selected_index, filtered_len);
    }

    pub fn process_key(
        &mut self,
        key: (KeyCode, KeyModifiers),
        filtered: &[usize],
    ) -> Option<SelectionResult> {
        process_key_input(
            key,
            &mut self.query,
            &mut self.selected_index,
            filtered,
            self.allow_custom,
        )
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

use crossterm::event::{KeyCode, KeyModifiers};
use fuzzy_matcher::clangd::ClangdMatcher;
use fuzzy_matcher::FuzzyMatcher;

pub fn filter_items_by_query(items: &[String], query: &str) -> Vec<usize> {
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    let matcher = ClangdMatcher::default();
    items
        .iter()
        .enumerate()
        .filter_map(|(i, s)| matcher.fuzzy_match(s, query).map(|_| i))
        .collect()
}

pub fn clamp_selection_index(selected: usize, filtered_len: usize) -> usize {
    if filtered_len == 0 {
        0
    } else if selected >= filtered_len {
        filtered_len - 1
    } else {
        selected
    }
}

pub fn process_key_input(
    key: (KeyCode, KeyModifiers),
    query: &mut String,
    selected_index: &mut usize,
    filtered_indices: &[usize],
    allow_custom: bool,
) -> Option<SelectionResult> {
    match key {
        (KeyCode::Char('c'), mods) if mods.contains(KeyModifiers::CONTROL) => {
            Some(SelectionResult::Cancelled)
        }
        (KeyCode::Char('u'), mods) if mods.contains(KeyModifiers::CONTROL) => {
            query.clear();
            *selected_index = 0;
            None
        }
        (KeyCode::Char(c), _) => {
            query.push(c);
            *selected_index = 0;
            None
        }
        (KeyCode::Backspace, _) => {
            query.pop();
            *selected_index = 0;
            None
        }
        (KeyCode::Down, _) => {
            *selected_index =
                clamp_selection_index(selected_index.saturating_add(1), filtered_indices.len());
            None
        }
        (KeyCode::Up, _) => {
            *selected_index =
                clamp_selection_index(selected_index.saturating_sub(1), filtered_indices.len());
            None
        }
        (KeyCode::Esc, _) => Some(SelectionResult::Cancelled),
        (KeyCode::Enter, _) => {
            if let Some(&idx) = filtered_indices.get(*selected_index) {
                Some(SelectionResult::Selected(idx))
            } else if allow_custom && !query.trim().is_empty() {
                Some(SelectionResult::Custom(query.trim().to_string()))
            } else {
                None
            }
        }
        _ => None,
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
    fn filter_items_by_query_empty_returns_all() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let result = filter_items_by_query(&items, "");
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn filter_items_by_query_matches_prefix() {
        let items = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let result = filter_items_by_query(&items, "ba");
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn filter_items_by_query_matches_subsequence() {
        let items = vec!["feature/foo".to_string(), "bug/bar".to_string()];
        let result = filter_items_by_query(&items, "f/foo");
        assert_eq!(result, vec![0]);
    }

    #[test]
    fn filter_items_by_query_no_match_returns_empty() {
        let items = vec!["foo".to_string(), "bar".to_string()];
        let result = filter_items_by_query(&items, "xyz");
        assert_eq!(result, Vec::<usize>::new());
    }

    #[test]
    fn clamp_selection_index_empty_filtered() {
        assert_eq!(clamp_selection_index(5, 0), 0);
    }

    #[test]
    fn clamp_selection_index_within_bounds() {
        assert_eq!(clamp_selection_index(1, 5), 1);
    }

    #[test]
    fn clamp_selection_index_exceeds_bounds() {
        assert_eq!(clamp_selection_index(5, 3), 2);
    }

    #[test]
    fn process_key_input_ctrl_c_returns_cancelled() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Char('c'), KeyModifiers::CONTROL),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, Some(SelectionResult::Cancelled));
    }

    #[test]
    fn process_key_input_ctrl_u_clears_query() {
        let mut query = "test".to_string();
        let mut selected = 2;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Char('u'), KeyModifiers::CONTROL),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(query, "");
        assert_eq!(selected, 0);
    }

    #[test]
    fn process_key_input_char_appends_to_query() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1];
        let result = process_key_input(
            (KeyCode::Char('a'), KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(query, "a");
        assert_eq!(selected, 0);
    }

    #[test]
    fn process_key_input_backspace_removes_last_char() {
        let mut query = "test".to_string();
        let mut selected = 0;
        let filtered = vec![0, 1];
        let result = process_key_input(
            (KeyCode::Backspace, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(query, "tes");
        assert_eq!(selected, 0);
    }

    #[test]
    fn process_key_input_down_increments_selection() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Down, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(selected, 1);
    }

    #[test]
    fn process_key_input_up_decrements_selection() {
        let mut query = String::new();
        let mut selected = 2;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Up, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(selected, 1);
    }

    #[test]
    fn process_key_input_up_at_zero_stays_zero() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Up, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
        assert_eq!(selected, 0);
    }

    #[test]
    fn process_key_input_esc_returns_cancelled() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1];
        let result = process_key_input(
            (KeyCode::Esc, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, Some(SelectionResult::Cancelled));
    }

    #[test]
    fn process_key_input_enter_selects_item() {
        let mut query = String::new();
        let mut selected = 1;
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, Some(SelectionResult::Selected(1)));
    }

    #[test]
    fn process_key_input_enter_creates_custom_when_allowed_and_query() {
        let mut query = "new-branch".to_string();
        let mut selected = 0;
        let filtered = vec![];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(
            result,
            Some(SelectionResult::Custom("new-branch".to_string()))
        );
    }

    #[test]
    fn process_key_input_enter_no_custom_when_not_allowed() {
        let mut query = "new-branch".to_string();
        let mut selected = 0;
        let filtered = vec![];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_input_enter_no_custom_when_query_empty() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
    }

    // Selector method tests

    #[test]
    fn selector_filter_delegates_to_filter_items_by_query() {
        let items = vec!["foo".to_string(), "bar".to_string(), "baz".to_string()];
        let selector = Selector::new(items.clone(), true);
        let result = selector.filter("ba");
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn selector_clamp_selection_delegates_to_clamp_selection_index() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let mut selector = Selector::new(items, true);
        // selected_index starts at 0; clamp_selection(3) delegates to
        // clamp_selection_index(0, 3) which returns 0 (within bounds).
        selector.clamp_selection(3);
        assert_eq!(selector.selected_index(), 0);
        // With filtered_len=0, selection stays at 0.
        selector.clamp_selection(0);
        assert_eq!(selector.selected_index(), 0);
    }

    #[test]
    fn selector_process_key_delegates() {
        let items = vec!["a".to_string(), "b".to_string()];
        let mut selector = Selector::new(items, true);
        let result = selector.process_key((KeyCode::Char('c'), KeyModifiers::CONTROL), &[0, 1]);
        assert_eq!(result, Some(SelectionResult::Cancelled));
        assert_eq!(selector.query(), "");
    }

    // process_key_input edge cases

    #[test]
    fn process_key_input_unknown_key_returns_none() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![0, 1];
        let result = process_key_input(
            (KeyCode::F(1), KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            true,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_input_enter_with_oob_index_returns_none() {
        let mut query = String::new();
        let mut selected = 5; // out of bounds for filtered_indices (len 3)
        let filtered = vec![0, 1, 2];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            false,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn process_key_input_enter_empty_query_no_custom_when_not_allowed() {
        let mut query = String::new();
        let mut selected = 0;
        let filtered = vec![];
        let result = process_key_input(
            (KeyCode::Enter, KeyModifiers::NONE),
            &mut query,
            &mut selected,
            &filtered,
            false,
        );
        assert_eq!(result, None);
    }
}
