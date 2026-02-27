use crate::colors::*;
use crate::tmux;

pub fn run() -> Result<(), String> {
    if !tmux::has_session() {
        println!("{ANSI_OVERLAY}No active cove session.{ANSI_RESET}");
        return Err(String::new());
    }

    let windows = tmux::list_windows()?;
    let home = std::env::var("HOME").unwrap_or_default();

    for w in &windows {
        let dir = w.pane_path.replace(&home, "~");
        if w.is_active {
            println!(
                "{ANSI_WHITE}{ANSI_BOLD}❯{ANSI_RESET} {ANSI_WHITE}{ANSI_BOLD}{}{ANSI_RESET}  {ANSI_SUBTEXT}{dir}{ANSI_RESET}",
                w.name
            );
        } else {
            println!(
                "  {ANSI_OVERLAY}{}{ANSI_RESET}  {ANSI_SURFACE}{dir}{ANSI_RESET}",
                w.name
            );
        }
    }

    Ok(())
}
