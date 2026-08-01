use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use fuzzy_matcher::clangd::ClangdMatcher;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Terminal;
use std::io;

pub enum Choice {
    Pick(usize),
    Type(String),
    Cancel,
}

type Term = Terminal<CrosstermBackend<io::Stdout>>;

pub fn fzf_pick(
    items: &[String],
    prompt: &str,
    header: &str,
    allow_create: bool,
) -> Result<Choice> {
    let mut terminal = setup_terminal()?;
    let matcher = ClangdMatcher::default();
    let mut query = String::new();
    let mut selected = 0usize;

    let result = loop {
        let filtered: Vec<usize> = items
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if query.is_empty() {
                    Some(i)
                } else {
                    matcher.fuzzy_match(s, &query).map(|_| i)
                }
            })
            .collect();

        if filtered.is_empty() {
            selected = 0;
        } else if selected >= filtered.len() {
            selected = filtered.len() - 1;
        }

        terminal
            .draw(|f| {
                let area = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(1),
                        Constraint::Min(0),
                    ])
                    .split(area);
                let input = Paragraph::new(format!("{prompt}{query}"))
                    .block(Block::default().borders(Borders::ALL));
                let hint = Paragraph::new(header).style(Style::default().fg(Color::DarkGray));
                f.render_widget(input, chunks[0]);
                f.render_widget(hint, chunks[1]);

                let list_items: Vec<ListItem> = filtered
                    .iter()
                    .map(|&i| ListItem::new(items[i].as_str()))
                    .collect();
                let mut ls = ListState::default();
                ls.select(Some(selected));
                let list = List::new(list_items)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(Color::Yellow));
                f.render_stateful_widget(list, chunks[2], &mut ls);
            })
            .context("Failed to draw terminal")?;

        if let Event::Key(k) = event::read().context("Failed to read key event")? { match k.code {
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                break Choice::Cancel;
            }
            KeyCode::Char('u') if k.modifiers.contains(KeyModifiers::CONTROL) => {
                query.clear();
            }
            KeyCode::Char(c) => query.push(c),
            KeyCode::Backspace => {
                query.pop();
            }
            KeyCode::Down
                if !filtered.is_empty() && selected + 1 < filtered.len() => {
                    selected += 1;
                }
            KeyCode::Up => {
                selected = selected.saturating_sub(1);
            }
            KeyCode::Esc => break Choice::Cancel,
            KeyCode::Enter => {
                if let Some(&idx) = filtered.get(selected) {
                    break Choice::Pick(idx);
                } else if allow_create && !query.trim().is_empty() {
                    break Choice::Type(query.trim().to_string());
                }
            }
            _ => {}
        } }
    };

    restore_terminal(&mut terminal)?;
    Ok(result)
}

fn setup_terminal() -> Result<Term> {
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    terminal.clear().context("Failed to clear terminal")?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Term) -> Result<()> {
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("Failed to leave alternate screen")?;
    disable_raw_mode().context("Failed to disable raw mode")?;
    Ok(())
}
