# Pause Loading Animation During Interactive Input

## Problem

The loading runner draws a ratatui animation while a background job runs. Git and
SSH may request interactive terminal input during that job. The animation can
continue redrawing over the prompt while the user types, making the prompt and
input difficult to see.

## Goal

When any terminal input is pending during a loading job, toggle the loading
animation off so the interactive process has an unobstructed terminal. If the
job is still running after input becomes idle, toggle the loading animation back
on after a 500 ms grace period.

## Scope

- Apply the behavior to any terminal input, not only password prompts.
- Preserve input for the interactive process; the loading runner must not read or
  consume events.
- Keep the existing minimum loading-display duration.
- Preserve terminal restoration on success, error, panic, setup failure, and
  animation failure.
- Do not change Git command execution or prompt handling.

## User-visible behavior

1. A loading job starts and the animation is rendered as it is today.
2. When terminal input is detected, the full loading screen is cleared and the
   input prompt becomes the only visible content. The animation remains paused
   while input is pending and must not redraw over the prompt.
3. Once input has been submitted and has been idle for 500 ms, the animation
   resumes if the job is still running.
4. If the job completes while the animation is paused, it does not resume; the
   terminal is restored normally.

## Design

### Input polling

Extend the internal `TerminalBackend` abstraction with a non-consuming input
poll operation. `RealTerminal` will implement it with crossterm's polling API.
Polling checks whether input is available but does not call `read`, so SSH/Git
continues to receive the input intended for its prompt.

The fake terminal backend will expose controllable pending-input state so the
state transitions can be tested deterministically.

### Loading state

The loading thread will track the toggled loading state and the last observed
input activity. On the first pending-input observation it toggles the animation
off, clears the full terminal once, and stops drawing animation frames so the
interactive prompt has focus. After input has been submitted and no input has
been pending for 500 ms, it toggles the animation back on and redraws if the job
has not completed. No intermediate animation frame may be drawn while paused.

The existing job thread, panic handling, minimum display timing, thread join,
and terminal restoration remain intact.

## Error handling

- Input polling errors are returned as loading-runner errors and terminal
  restoration follows the existing error path.
- A draw or clear error while pausing or resuming is returned and does not get
  swallowed.
- Job errors and panics retain their current behavior.
- No input is consumed by the animation thread.

## Testing strategy (TDD)

1. Add a failing test that verifies pending input toggles loading off, clears the
   full terminal, and prevents further animation frames while input is active.
2. Add a test that verifies 500 ms after input submission toggles loading back on
   while the job remains active.
3. Add a test that verifies completion while paused does not restart animation.
4. Retain and extend tests for terminal restoration on successful, failed, and
   panicking jobs, setup failure, and draw failure.
5. Add a real terminal smoke test with a long-running loading job and injected
   input, verifying that the animation disappears and returns without input
   being consumed.

## Out of scope

- Detecting or parsing password prompt text.
- Changing Git or SSH invocation, stdin, stdout, or stderr handling.
- Adding a user-configurable idle timeout.
- Changing selector, cleanup, or worktree behavior.
