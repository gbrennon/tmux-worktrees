use crate::core::error::{Error, Result};
use ratatui::{Frame, Terminal, backend::Backend};

pub(crate) trait TerminalBackend {
    type Term: Backend;

    fn setup(&self) -> Result<Terminal<Self::Term>>;

    fn draw_frame(
        &self,
        terminal: &mut Terminal<Self::Term>,
        f: impl FnOnce(&mut Frame),
    ) -> Result<()> {
        terminal
            .draw(f)
            .map(|_| ())
            .map_err(|e| Error::new(format!("Failed to draw terminal: {}", e)))
    }

    fn read_event(&self) -> Result<crossterm::event::Event>;

    fn restore(&self, terminal: &mut Terminal<Self::Term>) -> Result<()>;
}

pub(crate) struct RealTerminal;

impl TerminalBackend for RealTerminal {
    type Term = ratatui::backend::CrosstermBackend<std::io::Stdout>;

    fn setup(&self) -> Result<Terminal<Self::Term>> {
        use std::io::stdout;
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout());
        Terminal::new(backend)
            .map_err(|e| Error::new(format!("Failed to initialize terminal: {}", e)))
    }

    fn read_event(&self) -> Result<crossterm::event::Event> {
        crossterm::event::read().map_err(|e| Error::new(format!("Failed to read key event: {}", e)))
    }

    fn restore(&self, terminal: &mut Terminal<Self::Term>) -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
        terminal.show_cursor()?;
        Ok(())
    }
}
