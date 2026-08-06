use fuzzy_matcher::{FuzzyMatcher, clangd::ClangdMatcher};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};

use super::terminal_backend::{RealTerminal, TerminalBackend};

impl From<crossterm::event::KeyEvent> for Key {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        use crossterm::event::{KeyCode, KeyModifiers};
        match event {
            crossterm::event::KeyEvent {
                code: KeyCode::Char(c),
                modifiers: m,
                ..
            } if m.contains(KeyModifiers::CONTROL) => match c {
                'c' => Key::CtrlC,
                'u' => Key::CtrlU,
                _ => Key::Unknown,
            },
            crossterm::event::KeyEvent {
                code: KeyCode::Char(c),
                ..
            } => Key::Char(c),
            crossterm::event::KeyEvent {
                code: KeyCode::Backspace,
                ..
            } => Key::Backspace,
            crossterm::event::KeyEvent {
                code: KeyCode::Down,
                ..
            } => Key::Down,
            crossterm::event::KeyEvent {
                code: KeyCode::Up, ..
            } => Key::Up,
            crossterm::event::KeyEvent {
                code: KeyCode::Esc, ..
            } => Key::Esc,
            crossterm::event::KeyEvent {
                code: KeyCode::Enter,
                ..
            } => Key::Enter,
            _ => Key::Unknown,
        }
    }
}

#[cfg(test)]
use ratatui::backend::TestBackend;

use crate::{
    core::{
        error::{Error, Result},
        selector::{Key, SelectionResult, Selector},
    },
    presentation::selector_port::SelectorRunner,
};

pub struct RatatuiSelector;

impl RatatuiSelector {
    fn run_with_backend<B: TerminalBackend>(
        backend: &B,
        selector: &mut Selector,
        _filtered: &[usize],
        prompt: &str,
        header: &str,
    ) -> Result<SelectionResult> {
        let mut terminal = backend.setup()?;
        terminal
            .clear()
            .map_err(|e| Error::new(format!("Failed to clear terminal: {}", e)))?;
        let matcher = ClangdMatcher::default();

        let result = loop {
            let filtered: Vec<usize> = selector
                .items()
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    if selector.query().is_empty() {
                        Some(i)
                    } else {
                        matcher.fuzzy_match(s, selector.query()).map(|_| i)
                    }
                })
                .collect();

            selector.clamp_selection(filtered.len());

            backend.draw_frame(&mut terminal, |f| {
                let area = f.area();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(area);
                let input = Paragraph::new(format!("{prompt}{}", selector.query()))
                    .block(Block::default().borders(Borders::ALL));
                let hint = Paragraph::new(header).style(Style::default().fg(Color::DarkGray));
                f.render_widget(input, chunks[0]);
                f.render_widget(hint, chunks[1]);

                let list_items: Vec<ListItem> = filtered
                    .iter()
                    .map(|&i| ListItem::new(selector.items()[i].as_str()))
                    .collect();
                let mut ls = ListState::default();
                ls.select(Some(selector.selected_index()));
                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(list, chunks[2], &mut ls);
            })?;

            if let crossterm::event::Event::Key(k) = backend.read_event()?
                && let Some(result) = selector.process_key(Key::from(k), &filtered)
            {
                break result;
            }
        };

        backend.restore(&mut terminal)?;
        Ok(result)
    }
}

impl SelectorRunner for RatatuiSelector {
    fn run_selector(
        &self,
        selector: &mut Selector,
        filtered: &[usize],
        prompt: &str,
        header: &str,
    ) -> Result<SelectionResult> {
        Self::run_with_backend(&RealTerminal, selector, filtered, prompt, header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Frame, Terminal};
    use std::cell::RefCell;

    thread_local! {
        static EVENTS: RefCell<Vec<crossterm::event::Event>> =
            const { RefCell::new(Vec::new()) };
        static SETUP_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static DRAW_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static READ_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static RESTORE_FAILS: RefCell<bool> = const { RefCell::new(false) };
    }

    fn reset_fake_state() {
        EVENTS.with(|e| e.borrow_mut().clear());
        SETUP_FAILS.with(|f| *f.borrow_mut() = false);
        DRAW_FAILS.with(|f| *f.borrow_mut() = false);
        READ_FAILS.with(|f| *f.borrow_mut() = false);
        RESTORE_FAILS.with(|f| *f.borrow_mut() = false);
    }

    fn set_restore_fails(v: bool) {
        RESTORE_FAILS.with(|f| *f.borrow_mut() = v);
    }

    fn push_events(events: Vec<crossterm::event::Event>) {
        EVENTS.with(|e| {
            let mut q = e.borrow_mut();
            for ev in events.into_iter().rev() {
                q.push(ev);
            }
        });
    }

    fn set_setup_fails(v: bool) {
        SETUP_FAILS.with(|f| *f.borrow_mut() = v);
    }

    fn set_draw_fails(v: bool) {
        DRAW_FAILS.with(|f| *f.borrow_mut() = v);
    }

    fn set_read_fails(v: bool) {
        READ_FAILS.with(|f| *f.borrow_mut() = v);
    }

    fn key_event(code: crossterm::event::KeyCode) -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code,
            modifiers: crossterm::event::KeyModifiers::empty(),
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    fn key_enter() -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Enter)
    }

    fn key_esc() -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Esc)
    }

    fn key_char(c: char) -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Char(c))
    }

    fn key_down() -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Down)
    }

    fn key_up() -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Up)
    }

    fn key_ctrl_u() -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('u'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    fn key_ctrl_c() -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char('c'),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    struct FakeTerminal;

    impl TerminalBackend for FakeTerminal {
        type Term = TestBackend;

        fn setup(&self) -> Result<Terminal<Self::Term>> {
            if SETUP_FAILS.with(|f| *f.borrow()) {
                return Err(Error::new("Simulated setup failure"));
            }
            let backend = TestBackend::new(80, 24);
            Terminal::new(backend)
                .map_err(|e| Error::new(format!("Failed to initialize terminal: {}", e)))
        }

        fn draw_frame(
            &self,
            terminal: &mut Terminal<Self::Term>,
            f: impl FnOnce(&mut Frame),
        ) -> Result<()> {
            if DRAW_FAILS.with(|d| *d.borrow()) {
                return Err(Error::new("Simulated draw failure"));
            }
            terminal
                .draw(f)
                .map(|_| ())
                .map_err(|e| Error::new(format!("Failed to draw terminal: {}", e)))
        }

        fn read_event(&self) -> Result<crossterm::event::Event> {
            if READ_FAILS.with(|r| *r.borrow()) {
                return Err(Error::new("Simulated read failure"));
            }
            EVENTS.with(|events| {
                events
                    .borrow_mut()
                    .pop()
                    .ok_or_else(|| Error::new("No more events in fake queue"))
            })
        }

        fn restore(&self, _terminal: &mut Terminal<Self::Term>) -> Result<()> {
            if RESTORE_FAILS.with(|r| *r.borrow()) {
                return Err(Error::new("Simulated restore failure"));
            }
            Ok(())
        }
    }

    fn run_with_fake(
        selector: &mut Selector,
        prompt: &str,
        header: &str,
    ) -> Result<SelectionResult> {
        RatatuiSelector::run_with_backend(&FakeTerminal, selector, &[], prompt, header)
    }

    #[test]
    fn setup_fails_without_tty() {
        reset_fake_state();
        set_setup_fails(true);

        let mut selector = Selector::new(vec!["a".into(), "b".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert!(result.is_err());
    }

    #[test]
    fn selects_first_item_on_enter() {
        reset_fake_state();
        push_events(vec![key_enter()]);

        let mut selector = Selector::new(vec!["alpha".into(), "beta".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn cancels_on_escape() {
        reset_fake_state();
        push_events(vec![key_esc()]);

        let mut selector = Selector::new(vec!["a".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Cancelled);
    }

    #[test]
    fn custom_value_on_enter_with_empty_items() {
        reset_fake_state();
        push_events(vec![
            key_char('h'),
            key_char('e'),
            key_char('l'),
            key_char('l'),
            key_char('o'),
            key_enter(),
        ]);

        let mut selector = Selector::new(vec![], true);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Custom("hello".into()));
    }

    #[test]
    fn filters_items_and_selects_correct_index() {
        reset_fake_state();
        push_events(vec![key_char('b'), key_char('a'), key_enter()]);

        let mut selector = Selector::new(vec!["foo".into(), "bar".into(), "baz".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(1));
    }

    #[test]
    fn draw_error_propagates() {
        reset_fake_state();
        set_draw_fails(true);
        push_events(vec![key_enter()]);

        let mut selector = Selector::new(vec!["x".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert!(result.is_err());
    }

    #[test]
    fn read_error_propagates() {
        reset_fake_state();
        set_read_fails(true);

        let mut selector = Selector::new(vec!["x".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert!(result.is_err());
    }

    #[test]
    fn down_arrow_moves_selection_then_enter_selects_second() {
        reset_fake_state();
        push_events(vec![key_down(), key_enter()]);

        let mut selector = Selector::new(vec!["first".into(), "second".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(1));
    }

    #[test]
    fn up_arrow_wraps_and_stays_at_first() {
        reset_fake_state();
        push_events(vec![key_up(), key_enter()]);

        let mut selector = Selector::new(vec!["a".into(), "b".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn ctrl_u_clears_query() {
        reset_fake_state();
        push_events(vec![
            key_char('x'),
            key_char('y'),
            key_char('z'),
            key_ctrl_u(),
            key_enter(),
        ]);

        let mut selector = Selector::new(vec!["apple".into(), "banana".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn backspace_erases_last_char() {
        reset_fake_state();
        push_events(vec![
            key_char('f'),
            key_event(crossterm::event::KeyCode::Backspace),
            key_enter(),
        ]);

        let mut selector = Selector::new(vec!["foo".into(), "bar".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn ctrl_c_cancels() {
        reset_fake_state();
        push_events(vec![key_ctrl_c()]);

        let mut selector = Selector::new(vec!["a".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Cancelled);
    }

    #[test]
    fn unknown_key_ignored_then_enter_selects() {
        reset_fake_state();
        push_events(vec![
            key_event(crossterm::event::KeyCode::F(1)),
            key_enter(),
        ]);

        let mut selector = Selector::new(vec!["item".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn restore_failure_propagates_error() {
        reset_fake_state();
        set_restore_fails(true);
        push_events(vec![key_enter()]);

        let mut selector = Selector::new(vec!["x".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert!(result.is_err());
    }
}
