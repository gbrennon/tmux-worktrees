use std::io::IsTerminal;

use tmux_worktrees::pipeline;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let ctx = pipeline::Ctx::default();
    let cmd = args.get(1).map(String::as_str).unwrap_or("choose");
    let interactive = matches!(cmd, "choose" | "cleanup");

    let result = if interactive && !std::io::stdin().is_terminal() {
        match pipeline::spawn_in_popup(&ctx, &args) {
            Ok(()) => Ok(()),
            Err(_) => pipeline::dispatch(&ctx, &args),
        }
    } else {
        pipeline::dispatch(&ctx, &args)
    };

    if let Err(e) = result {
        let _ = ctx.tmux.show_error(&format!("{e:#}"));
    }
    std::process::exit(0);
}
