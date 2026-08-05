fn main() {
    let args: Vec<String> = std::env::args().collect();
    tmux_worktrees::build_cli().run(&args);
    std::process::exit(0);
}
