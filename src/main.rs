use std::io::IsTerminal;

use tmux_worktrees::{
    app,
    infrastructure::{git_executor::GitExecutor, tmux_executor::TmuxExecutor},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tmux = TmuxExecutor::default();
    let git = GitExecutor::default();
    let cmd = args.get(1).map(String::as_str).unwrap_or("choose");
    let interactive = matches!(cmd, "choose" | "cleanup");

    let result = if interactive && !std::io::stdin().is_terminal() {
        match app::spawn_in_popup(&tmux, &git, &args) {
            Ok(()) => Ok(()),
            Err(_) => app::dispatch(&tmux, &git, &args),
        }
    } else {
        app::dispatch(&tmux, &git, &args)
    };

    if let Err(e) = result {
        let _ = tmux.show_error(&format!("{e:#}"));
    }
    std::process::exit(0);
}
