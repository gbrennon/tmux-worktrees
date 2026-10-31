use fuzzy_matcher::{FuzzyMatcher, clangd::ClangdMatcher};
use ratatui::{
    layout::{Constraint, Direction, Layout, Position},
    style::{Color, Style},
    text::Text,
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
                'j' => Key::Down,
                'k' => Key::Up,
                _ => Key::Unknown,
            },
            crossterm::event::KeyEvent {
                code: KeyCode::Char(' '),
                modifiers: KeyModifiers::NONE,
                ..
            } => Key::Space,
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
                let input_text = format!("{prompt}{}", selector.query());
                let input = Paragraph::new(input_text.clone())
                    .style(Style::default().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Cyan)),
                    );
                let hint_text = if selector.is_multi_select() {
                    format!("{header} Space to toggle, Enter to continue, Esc to cancel")
                } else {
                    header.to_string()
                };
                let hint = Paragraph::new(hint_text).style(Style::default().fg(Color::DarkGray));
                f.render_widget(input, chunks[0]);
                f.render_widget(hint, chunks[1]);

                let cursor_x = chunks[0]
                    .x
                    .saturating_add(1)
                    .saturating_add(Text::raw(input_text).width() as u16)
                    .min(chunks[0].right().saturating_sub(1));
                f.set_cursor_position(Position::new(cursor_x, chunks[0].y.saturating_add(1)));

                let list_items: Vec<ListItem> = filtered
                    .iter()
                    .map(|&i| {
                        if selector.is_multi_select() {
                            let marker = if selector.is_checked(i) { '*' } else { ' ' };
                            ListItem::new(format!("[{marker}] {}", selector.items()[i]))
                        } else {
                            ListItem::new(selector.items()[i].as_str())
                        }
                    })
                    .collect();
                let mut ls = ListState::default();
                ls.select(Some(selector.selected_index()));
                let list = List::new(list_items)
                    .style(Style::default().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(Color::Blue)),
                    )
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
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
    use ratatui::{Frame, Terminal, buffer::Buffer, layout::Position};
    use std::cell::RefCell;

    thread_local! {
        static EVENTS: RefCell<Vec<crossterm::event::Event>> =
            const { RefCell::new(Vec::new()) };
        static LAST_CURSOR_POSITION: RefCell<Option<Position>> = const { RefCell::new(None) };
        static LAST_BUFFER: RefCell<Option<Buffer>> = const { RefCell::new(None) };
        static SETUP_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static DRAW_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static READ_FAILS: RefCell<bool> = const { RefCell::new(false) };
        static RESTORE_FAILS: RefCell<bool> = const { RefCell::new(false) };
    }

    fn reset_fake_state() {
        EVENTS.with(|e| e.borrow_mut().clear());
        LAST_CURSOR_POSITION.with(|p| *p.borrow_mut() = None);
        LAST_BUFFER.with(|b| *b.borrow_mut() = None);
        SETUP_FAILS.with(|f| *f.borrow_mut() = false);
        DRAW_FAILS.with(|f| *f.borrow_mut() = false);
        READ_FAILS.with(|f| *f.borrow_mut() = false);
        RESTORE_FAILS.with(|f| *f.borrow_mut() = false);
    }

    fn last_buffer_text() -> String {
        LAST_BUFFER.with(|buffer| {
            let buffer = buffer.borrow();
            let buffer = buffer.as_ref().expect("fake terminal did not draw");
            (0..buffer.area.height)
                .flat_map(|y| {
                    (0..buffer.area.width).map(move |x| buffer[(x, y)].symbol().to_string())
                })
                .collect()
        })
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

    fn key_from_event(event: crossterm::event::Event) -> Key {
        match event {
            crossterm::event::Event::Key(event) => Key::from(event),
            _ => Key::Unknown,
        }
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

    fn key_ctrl_j() -> crossterm::event::Event {
        key_ctrl_char('j')
    }

    fn key_ctrl_k() -> crossterm::event::Event {
        key_ctrl_char('k')
    }

    fn key_up() -> crossterm::event::Event {
        key_event(crossterm::event::KeyCode::Up)
    }

    fn key_ctrl_char(c: char) -> crossterm::event::Event {
        crossterm::event::Event::Key(crossterm::event::KeyEvent {
            code: crossterm::event::KeyCode::Char(c),
            modifiers: crossterm::event::KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        })
    }

    fn key_ctrl_u() -> crossterm::event::Event {
        key_ctrl_char('u')
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
            let result = terminal
                .draw(f)
                .map(|_| ())
                .map_err(|e| Error::new(format!("Failed to draw terminal: {}", e)));
            if result.is_ok() {
                LAST_CURSOR_POSITION.with(|p| {
                    *p.borrow_mut() = Some(terminal.backend().cursor_position());
                });
                LAST_BUFFER.with(|b| {
                    *b.borrow_mut() = Some(terminal.backend().buffer().clone());
                });
            }
            result
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
    fn cursor_is_positioned_after_prompt_and_query() {
        reset_fake_state();
        push_events(vec![key_char('x'), key_esc()]);

        let mut selector = Selector::new(vec!["x".into()], false);
        run_with_fake(&mut selector, "> ", "hint").unwrap();

        let cursor = LAST_CURSOR_POSITION.with(|p| *p.borrow());
        assert_eq!(cursor, Some(Position::new(4, 1)));
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
    fn multi_selects_two_items_with_space_and_enter() {
        reset_fake_state();
        push_events(vec![key_char(' '), key_down(), key_char(' '), key_enter()]);

        let mut selector = Selector::new_multi(vec!["first".into(), "second".into()]);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::SelectedMany(vec![0, 1]));
    }

    #[test]
    fn multi_select_renders_unchecked_markers_and_hint() {
        reset_fake_state();
        push_events(vec![key_esc()]);

        let mut selector = Selector::new_multi(vec!["first".into(), "second".into()]);
        run_with_fake(&mut selector, "> ", "Select worktrees").unwrap();
        let text = last_buffer_text();

        assert!(text.contains("[ ] first"));
        assert!(text.contains("[ ] second"));
        assert!(
            text.contains("Select worktrees Space to toggle, Enter to continue, Esc to cancel")
        );
    }

    #[test]
    fn multi_select_renders_checked_marker_after_space() {
        reset_fake_state();
        push_events(vec![key_char(' '), key_enter()]);

        let mut selector = Selector::new_multi(vec!["first".into(), "second".into()]);
        run_with_fake(&mut selector, "> ", "hint").unwrap();
        let text = last_buffer_text();

        assert!(text.contains("[*] first"));
        assert!(text.contains("[ ] second"));
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
    fn ctrl_j_moves_selection_down() {
        reset_fake_state();
        push_events(vec![key_ctrl_j(), key_enter()]);

        let mut selector = Selector::new(vec!["first".into(), "second".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(1));
    }

    #[test]
    fn ctrl_k_moves_selection_up() {
        reset_fake_state();
        push_events(vec![key_down(), key_ctrl_k(), key_enter()]);

        let mut selector = Selector::new(vec!["first".into(), "second".into()], false);
        let result = run_with_fake(&mut selector, "> ", "hint");
        assert_eq!(result.unwrap(), SelectionResult::Selected(0));
    }

    #[test]
    fn plain_j_and_k_remain_query_input() {
        assert_eq!(key_from_event(key_char('j')), Key::Char('j'));
        assert_eq!(key_from_event(key_char('k')), Key::Char('k'));
        assert_eq!(key_from_event(key_char(' ')), Key::Space);
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
