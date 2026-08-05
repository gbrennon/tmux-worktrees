use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use ratatui::Terminal;

use crate::core::error::{Error, Result};
use crate::presentation::loading_port::LoadingRunner;
use crate::presentation::terminal_backend::{RealTerminal, TerminalBackend};
use crate::presentation::widgets::loading::LoadingIndicator;

const FRAME_DELAY: Duration = Duration::from_millis(100);

pub struct RatatuiLoading;

impl RatatuiLoading {
    fn run_with_backend<B, F>(backend: B, label: String, job: F) -> Result<()>
    where
        B: TerminalBackend + Send + Sync + 'static,
        B::Term: Send,
        F: FnOnce() -> Result<()>,
    {
        let mut terminal = backend.setup()?;
        terminal
            .clear()
            .map_err(|e| Error::new(format!("Failed to clear terminal: {}", e)))?;

        let backend = Arc::new(backend);
        let thread_backend = Arc::clone(&backend);
        let done = Arc::new(AtomicBool::new(false));
        let thread_done = Arc::clone(&done);

        let handle = thread::spawn(move || -> Result<Terminal<B::Term>> {
            let mut indicator = LoadingIndicator::new(label);
            loop {
                indicator.tick();
                thread_backend.draw_frame(&mut terminal, |f| indicator.render(f, f.area()))?;
                if thread_done.load(Ordering::Relaxed) {
                    break;
                }
                thread::sleep(FRAME_DELAY);
            }
            Ok(terminal)
        });

        let job_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job))
            .unwrap_or_else(|_| Err(Error::new("Loading job panicked")));

        done.store(true, Ordering::Relaxed);
        let mut terminal = match handle.join() {
            Ok(Ok(terminal)) => terminal,
            Ok(Err(e)) => return Err(e),
            Err(_) => return Err(Error::new("Loading animation thread panicked")),
        };

        backend.restore(&mut terminal)?;
        job_result
    }
}

impl LoadingRunner for RatatuiLoading {
    fn run_loading(&self, label: &str, job: Box<dyn FnOnce() -> Result<()> + '_>) -> Result<()> {
        Self::run_with_backend(RealTerminal, label.to_string(), job)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{Frame, backend::TestBackend};

    #[derive(Default)]
    struct FakeState {
        setup_fails: AtomicBool,
        draw_fails: AtomicBool,
        restore_called: AtomicBool,
        draw_count: std::sync::atomic::AtomicUsize,
    }

    #[derive(Clone)]
    struct FakeTerminal {
        state: Arc<FakeState>,
    }

    impl FakeTerminal {
        fn new() -> Self {
            Self {
                state: Arc::new(FakeState::default()),
            }
        }
    }

    impl TerminalBackend for FakeTerminal {
        type Term = TestBackend;

        fn setup(&self) -> Result<Terminal<Self::Term>> {
            if self.state.setup_fails.load(Ordering::Relaxed) {
                return Err(Error::new("Simulated setup failure"));
            }
            Terminal::new(TestBackend::new(80, 24))
                .map_err(|e| Error::new(format!("Failed to initialize terminal: {e}")))
        }

        fn draw_frame(
            &self,
            terminal: &mut Terminal<Self::Term>,
            f: impl FnOnce(&mut Frame),
        ) -> Result<()> {
            if self.state.draw_fails.load(Ordering::Relaxed) {
                return Err(Error::new("Simulated draw failure"));
            }
            self.state.draw_count.fetch_add(1, Ordering::Relaxed);
            terminal
                .draw(f)
                .map(|_| ())
                .map_err(|e| Error::new(format!("Failed to draw terminal: {e}")))
        }

        fn read_event(&self) -> Result<crossterm::event::Event> {
            unimplemented!("loading runner never reads events")
        }

        fn restore(&self, _terminal: &mut Terminal<Self::Term>) -> Result<()> {
            self.state.restore_called.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    fn run(label: &str, job: impl FnOnce() -> Result<()>) -> (Result<()>, FakeTerminal) {
        let fake = FakeTerminal::new();
        let result = RatatuiLoading::run_with_backend(fake.clone(), label.to_string(), job);
        (result, fake)
    }

    #[test]
    fn runs_job_returns_result_and_restores() {
        let (result, fake) = run("loading", || Ok(()));
        assert!(result.is_ok());
        assert!(fake.state.restore_called.load(Ordering::Relaxed));
        assert!(
            fake.state.draw_count.load(Ordering::Relaxed) >= 1,
            "animation should draw at least one frame"
        );
    }

    #[test]
    fn propagates_job_error_but_restores_terminal() {
        let (result, fake) = run("loading", || Err(Error::new("boom")));
        assert!(result.is_err());
        assert!(fake.state.restore_called.load(Ordering::Relaxed));
    }

    #[test]
    fn setup_failure_propagates_without_restore() {
        let fake = FakeTerminal::new();
        fake.state.setup_fails.store(true, Ordering::Relaxed);
        let result = RatatuiLoading::run_with_backend(fake.clone(), "x".into(), || Ok(()));
        assert!(result.is_err());
        assert!(!fake.state.restore_called.load(Ordering::Relaxed));
    }

    #[test]
    fn draw_failure_propagates() {
        let fake = FakeTerminal::new();
        fake.state.draw_fails.store(true, Ordering::Relaxed);
        let result = RatatuiLoading::run_with_backend(fake.clone(), "x".into(), || Ok(()));
        assert!(result.is_err());
    }

    #[test]
    fn job_panic_is_caught_and_terminal_restored() {
        let (result, fake) = run("loading", || panic!("job exploded"));
        assert!(result.is_err());
        assert!(fake.state.restore_called.load(Ordering::Relaxed));
    }
}
