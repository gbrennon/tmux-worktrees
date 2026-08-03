use tmux_worktrees::{
    infrastructure::{git_executor::GitExecutor, tmux_executor::TmuxExecutor},
    presentation::cli::Cli,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let tmux = Box::new(TmuxExecutor::default());
    let git = Box::new(GitExecutor::default());
    let cli = Cli::new(tmux, git);
    cli.run(&args);
    std::process::exit(0);
}
