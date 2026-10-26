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
        let min_display = backend.min_display();
        let start = std::time::Instant::now();

        let mut terminal = backend.setup()?;
        backend.clear(&mut terminal)?;

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

        let elapsed = start.elapsed();
        if elapsed < min_display {
            thread::sleep(min_display - elapsed);
        }

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
    use std::sync::atomic::AtomicUsize;

    use super::*;
    use ratatui::{Frame, backend::TestBackend};

    #[derive(Default)]
    struct FakeState {
        setup_fails: AtomicBool,
        draw_fails: AtomicBool,
        restore_called: AtomicBool,
        draw_count: AtomicUsize,
        input_pending: AtomicBool,
        input_poll_fails: AtomicBool,
        input_poll_observed: AtomicBool,
        clear_count: AtomicUsize,
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

        fn set_input_pending(&self, pending: bool) {
            self.state.input_pending.store(pending, Ordering::Relaxed);
        }

        fn set_input_poll_fails(&self, fails: bool) {
            self.state.input_poll_fails.store(fails, Ordering::Relaxed);
        }

        fn clear_count(&self) -> usize {
            self.state.clear_count.load(Ordering::Relaxed)
        }

        fn draw_count(&self) -> usize {
            self.state.draw_count.load(Ordering::Relaxed)
        }

        fn input_poll_observed(&self) -> bool {
            self.state.input_poll_observed.load(Ordering::Relaxed)
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

        fn input_pending(&self) -> Result<bool> {
            self.state
                .input_poll_observed
                .store(true, Ordering::Relaxed);
            if self.state.input_poll_fails.load(Ordering::Relaxed) {
                return Err(Error::new("Simulated input polling failure"));
            }
            Ok(self.state.input_pending.load(Ordering::Relaxed))
        }

        fn clear(&self, terminal: &mut Terminal<Self::Term>) -> Result<()> {
            self.state.clear_count.fetch_add(1, Ordering::Relaxed);
            terminal
                .clear()
                .map_err(|e| Error::new(format!("Failed to clear terminal: {e}")))
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

    fn wait_for(mut condition: impl FnMut() -> bool) -> bool {
        let deadline = std::time::Instant::now() + Duration::from_secs(1);
        while !condition() {
            if std::time::Instant::now() >= deadline {
                return false;
            }
            thread::yield_now();
        }
        true
    }

    #[test]
    fn fake_backend_exposes_input_polling_and_clear() {
        let fake = FakeTerminal::new();
        assert!(!fake.input_pending().unwrap());
        fake.set_input_pending(true);
        assert!(fake.input_pending().unwrap());

        let mut terminal = fake.setup().unwrap();
        fake.clear(&mut terminal).unwrap();
        assert_eq!(fake.clear_count(), 1);
    }

    #[test]
    fn pending_input_clears_terminal_and_pauses_animation() {
        let fake = FakeTerminal::new();
        fake.set_input_pending(true);
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let runner_fake = fake.clone();
        let handle = thread::spawn(move || {
            RatatuiLoading::run_with_backend(runner_fake, "loading".into(), || {
                started_tx.send(()).unwrap();
                finish_rx.recv().unwrap();
                Ok(())
            })
        });

        started_rx.recv().unwrap();
        let cleared_for_pending = wait_for(|| fake.clear_count() >= 2);
        if !cleared_for_pending {
            finish_tx.send(()).unwrap();
            handle.join().unwrap().unwrap();
            panic!("pending input should clear the terminal");
        }
        assert_eq!(
            fake.clear_count(),
            2,
            "pending input should clear the terminal"
        );
        assert_eq!(
            fake.draw_count(),
            0,
            "animation must stay paused while input is pending"
        );
        finish_tx.send(()).unwrap();
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn idle_input_resumes_animation_after_500ms() {
        let fake = FakeTerminal::new();
        fake.set_input_pending(true);
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let runner_fake = fake.clone();
        let handle = thread::spawn(move || {
            RatatuiLoading::run_with_backend(runner_fake, "loading".into(), || {
                started_tx.send(()).unwrap();
                finish_rx.recv().unwrap();
                Ok(())
            })
        });

        started_rx.recv().unwrap();
        let cleared_for_pending = wait_for(|| fake.clear_count() >= 2);
        if !cleared_for_pending {
            finish_tx.send(()).unwrap();
            handle.join().unwrap().unwrap();
            panic!("pending input should clear the terminal");
        }
        assert_eq!(
            fake.draw_count(),
            0,
            "animation must stay paused while input is pending"
        );
        fake.set_input_pending(false);
        thread::sleep(Duration::from_millis(250));
        assert_eq!(
            fake.draw_count(),
            0,
            "animation must remain paused during the idle grace period"
        );
        thread::sleep(Duration::from_millis(350));
        assert!(
            fake.draw_count() > 0,
            "animation should resume after the idle grace period"
        );
        finish_tx.send(()).unwrap();
        handle.join().unwrap().unwrap();
    }

    #[test]
    fn completion_while_paused_does_not_resume_animation() {
        let fake = FakeTerminal::new();
        fake.set_input_pending(true);
        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let runner_fake = fake.clone();
        let handle = thread::spawn(move || {
            RatatuiLoading::run_with_backend(runner_fake, "loading".into(), || {
                started_tx.send(()).unwrap();
                finish_rx.recv().unwrap();
                Ok(())
            })
        });

        started_rx.recv().unwrap();
        let cleared_for_pending = wait_for(|| fake.clear_count() >= 2);
        if !cleared_for_pending {
            finish_tx.send(()).unwrap();
            handle.join().unwrap().unwrap();
            panic!("pending input should clear the terminal");
        }
        assert_eq!(fake.draw_count(), 0);
        finish_tx.send(()).unwrap();
        handle.join().unwrap().unwrap();
        assert_eq!(
            fake.draw_count(),
            0,
            "completion must not resume a paused animation"
        );
        assert_eq!(
            fake.clear_count(),
            2,
            "pending input should clear the terminal"
        );
    }

    #[test]
    fn input_polling_error_propagates_and_restores_terminal() {
        let fake = FakeTerminal::new();
        fake.set_input_poll_fails(true);
        let (finish_tx, finish_rx) = std::sync::mpsc::channel();
        let runner_fake = fake.clone();
        let handle = thread::spawn(move || {
            RatatuiLoading::run_with_backend(runner_fake, "loading".into(), || {
                finish_rx.recv().unwrap();
                Ok(())
            })
        });

        let polled = wait_for(|| fake.input_poll_observed());
        finish_tx.send(()).unwrap();
        let result = handle.join().unwrap();
        assert!(
            polled,
            "the loading loop should poll input before the job completes"
        );
        assert!(result.is_err());
        assert!(fake.state.restore_called.load(Ordering::Relaxed));
    }

    #[test]
    fn runs_job_returns_result_and_restores() {
        let (result, fake) = run("loading", || Ok(()));
        assert!(result.is_ok());
        assert_eq!(fake.clear_count(), 1);
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
