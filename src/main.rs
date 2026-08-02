fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ctx = tmux_worktrees::pipeline::Ctx::default();
    tmux_worktrees::pipeline::run(&ctx, &args);
}
