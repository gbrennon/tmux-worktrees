# Pause Loading Animation During Interactive Input Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatically toggle the loading animation off while terminal input is pending and back on after 500 ms of input inactivity while the job remains active.

**Architecture:** Extend the internal `TerminalBackend` with non-consuming input polling. The loading thread owns a toggled animation state: it clears the terminal and stops drawing when polling observes input, then redraws after a 500 ms idle period. The job thread, Git execution, and terminal input stream remain unchanged.

**Tech Stack:** Rust 2024, crossterm, ratatui, `std::thread`, `std::sync::atomic`, existing `TerminalBackend` and loading-runner test fakes.

## Global Constraints

- Apply the behavior to any terminal input, not only password prompts.
- The loading runner must not read or consume terminal events.
- Use a fixed 500 ms input-idle grace period; do not add configuration.
- Preserve the existing minimum loading-display duration.
- Preserve terminal restoration on success, error, panic, setup failure, and animation failure.
- Do not change Git command execution or prompt handling.

---

## File Map

- Modify `src/presentation/terminal_backend.rs`: add non-consuming input polling and a testable terminal-clear operation, with crossterm-backed behavior for the real terminal.
- Modify `src/presentation/ratatui_loading.rs`: implement the automatic animation toggle, terminal clearing, idle timing, and error propagation.
- Modify `src/presentation/ratatui_loading.rs` tests: extend the fake backend and add deterministic state-transition tests.
- Modify `tests/presentation/ratatui_loading_test.rs` only if the existing integration-test layout supports a real terminal smoke test without changing production APIs.

## Task 1: Add the non-consuming polling contract

**Files:**
- Modify: `src/presentation/terminal_backend.rs`
- Test: `src/presentation/ratatui_loading.rs` fake backend implementation

**Interfaces:**
- Produces `TerminalBackend::input_pending(&self) -> Result<bool>` with a default implementation returning `Ok(false)`.
- Produces `TerminalBackend::clear(&self, terminal: &mut Terminal<Self::Term>) -> Result<()>` with a default implementation delegating to `terminal.clear()`.
- `RealTerminal::input_pending` calls `crossterm::event::poll(Duration::ZERO)` and does not call `read`.
- The loading fake overrides `input_pending` and `clear` to expose deterministic state to tests; the selector fake uses the safe defaults.

- [ ] **Step 1: Add a failing compile/test expectation for polling and clearing**

Add fake-backend fields such as `input_pending: AtomicBool` and `clear_count: AtomicUsize`, plus test helpers to control and inspect them. Add the test-facing call sites before implementing the trait methods so the missing contract is visible.

- [ ] **Step 2: Run the focused tests and verify failure**

Run:

```bash
cargo test presentation::ratatui_loading -- --test-threads=1
```

Expected: compilation failure because `TerminalBackend` has no input-poll operation.

- [ ] **Step 3: Add the trait method and real implementation**

Add these default trait methods:

```rust
fn input_pending(&self) -> Result<bool> {
    Ok(false)
}

fn clear(&self, terminal: &mut Terminal<Self::Term>) -> Result<()> {
    terminal
        .clear()
        .map_err(|e| Error::new(format!("Failed to clear terminal: {e}")))
}
```

Override `input_pending` in `RealTerminal`:

```rust
fn input_pending(&self) -> Result<bool> {
    crossterm::event::poll(Duration::ZERO)
        .map_err(|e| Error::new(format!("Failed to poll terminal input: {e}")))
}
```

Override `input_pending` and `clear` in the loading fake, recording clear calls and returning its controlled pending-input state.

- [ ] **Step 4: Run focused tests**

Run:

```bash
cargo test presentation::ratatui_loading -- --test-threads=1
```

Expected: existing tests pass.

- [ ] **Step 5: Commit the contract**

```bash
git add src/presentation/terminal_backend.rs src/presentation/ratatui_loading.rs
git commit -m "feat(loading): poll terminal input without consuming it"
```

## Task 2: Add failing loading-state tests

**Files:**
- Modify: `src/presentation/ratatui_loading.rs`

**Interfaces:**
- Uses the fake backend's controlled `input_pending` state and draw counter.
- Keeps `RatatuiLoading::run_with_backend` as the unit-test seam.

- [ ] **Step 1: Write the pending-input pause test**

Create a long-running job synchronized with a channel or atomic flag. Set fake input pending, run the loading runner, and assert that the backend records a clear operation and that draw calls stop while input remains pending.

- [ ] **Step 2: Write the 500 ms resume test**

Start with input pending, assert that the full terminal is cleared and no animation frames are drawn while input remains pending, then mark input submitted/idle, keep the job running for more than 500 ms, and assert that draw calls occur again only after the idle grace period.

- [ ] **Step 3: Write the completion-while-paused test**

Keep input pending, complete the job, and assert that no post-completion resume draw occurs.

- [ ] **Step 4: Write input-polling-error propagation test**

Configure the fake backend to return an input-polling error and assert that the loading runner returns an error and restores the terminal.

- [ ] **Step 5: Run tests to verify the new tests fail**

Run:

```bash
cargo test presentation::ratatui_loading -- --test-threads=1
```

Expected: the new state-transition tests fail because the loading loop does not yet poll or toggle animation.

## Task 3: Implement automatic animation toggling

**Files:**
- Modify: `src/presentation/ratatui_loading.rs`

**Interfaces:**
- Add a named constant such as `INPUT_IDLE_GRACE: Duration = Duration::from_millis(500)`.
- Continue using the existing `done` atomic and loading thread.
- Propagate `input_pending`, `clear`, and draw errors through the loading thread result.

- [ ] **Step 1: Add animation state and timestamps**

Track `animation_enabled` and `last_input_seen: Option<Instant>` inside the loading thread. Initialize animation enabled.

- [ ] **Step 2: Toggle animation off on pending input**

Each loop iteration calls `input_pending`. When it returns true, record the current time. If animation is enabled, clear the full terminal once and set animation disabled. Do not read any event, and do not draw any animation frame while disabled so the interactive prompt remains the only focused content.

- [ ] **Step 3: Toggle animation on after idle input**

When input has been submitted, is no longer pending, and the last observed input is at least 500 ms old, set animation enabled and draw the next loading frame. If the job is done, do not resume.

- [ ] **Step 4: Preserve existing timing and shutdown behavior**

Keep the existing minimum-display sleep, done flag, thread join, panic conversion, and terminal restore path. Ensure the loop still sleeps for `FRAME_DELAY` and does not busy-spin while paused.

- [ ] **Step 5: Run focused tests**

Run:

```bash
cargo test presentation::ratatui_loading -- --test-threads=1
```

Expected: all loading-runner tests pass, including pause, resume, completion, and error cases.

- [ ] **Step 6: Commit the implementation**

```bash
git add src/presentation/ratatui_loading.rs
 git commit -m "fix(loading): hide animation during terminal input"
```

## Task 4: Add terminal smoke coverage and run the full gate

**Files:**
- Modify: `src/presentation/ratatui_loading.rs` tests or `tests/presentation/ratatui_loading_test.rs`, depending on the existing test harness.
- Modify: documentation only if the user-visible loading behavior needs explanation.

**Interfaces:**
- Smoke coverage must use a long-running job and verify the animation backend clears and redraws without consuming input.
- Do not add production-only test hooks solely to make the smoke test pass.

- [ ] **Step 1: Add the smoke test**

Use the existing fake terminal backend if a real terminal cannot inject input safely. The test must model pending input, idle time, and job completion and assert the observable clear/draw sequence.

- [ ] **Step 2: Run the complete project checks**

Run:

```bash
just test
just fmt-check
just lint
just test-release-scripts
git diff --check
```

Expected: all commands exit successfully.

- [ ] **Step 3: Inspect the final diff**

Run:

```bash
git diff --stat codeberg/main...HEAD
git diff --check
```

Confirm only the loading backend, loading runner, and relevant tests/docs changed.

- [ ] **Step 4: Commit final test/doc changes**

```bash
git add src/presentation/terminal_backend.rs src/presentation/ratatui_loading.rs tests/presentation/ratatui_loading_test.rs docs
git commit -m "test(loading): cover input-aware animation state"
```
