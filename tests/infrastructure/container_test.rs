use tmux_worktrees::infrastructure::container::Container;

#[test]
fn container_builds_cli_without_panicking() {
    let _cli = Container::build_cli();
}
