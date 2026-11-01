use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
};
use throbber_widgets_tui::{BRAILLE_SIX, Throbber, ThrobberState, WhichUse};

pub struct LoadingIndicator<'a> {
    throbber: Throbber<'a>,
    state: ThrobberState,
}

impl<'a> LoadingIndicator<'a> {
    pub fn new(label: impl Into<Span<'a>>) -> Self {
        Self {
            throbber: Throbber::default()
                .label(label.into())
                .throbber_set(BRAILLE_SIX)
                .use_type(WhichUse::Spin)
                .style(Style::default().fg(Color::Cyan))
                .throbber_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            state: ThrobberState::default(),
        }
    }

    pub fn tick(&mut self) {
        self.state.calc_next();
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let width = self.throbber.to_line(&self.state).width() as u16;
        let x = area.x + area.width.saturating_sub(width) / 2;
        let y = area.y + area.height.saturating_sub(1) / 2;
        let line_area = Rect::new(x, y, width.max(1), 1);
        frame.render_stateful_widget(self.throbber.clone(), line_area, &mut self.state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn rendered(label: &str) -> String {
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        let mut indicator = LoadingIndicator::new(label);
        terminal.draw(|f| indicator.render(f, f.area())).unwrap();
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect()
    }

    #[test]
    fn render_draws_label() {
        let buffer = rendered("Fetching origin");
        assert!(buffer.contains("Fetching origin"));
    }

    #[test]
    fn render_draws_throbber_symbol() {
        let buffer = rendered("Fetching origin");
        let has_braille = buffer.chars().any(|c| matches!(c as u32, 0x2800..=0x28ff));
        assert!(
            has_braille,
            "expected a braille throbber symbol in buffer: {buffer:?}"
        );
    }

    #[test]
    fn tick_advances_animation_frame() {
        let mut indicator = LoadingIndicator::new("x");
        let before = indicator.state.index();
        indicator.tick();
        assert_ne!(indicator.state.index(), before);
    }
}
