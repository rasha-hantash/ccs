use crate::colors::*;
use crate::tmux;

pub fn run() -> Result<(), String> {
    if !tmux::has_session() {
        return Err(format!(
            "{ANSI_OVERLAY}No active cove session.{ANSI_RESET} Run {ANSI_PEACH}cove{ANSI_RESET} to create one."
        ));
    }

    if tmux::is_inside_tmux() {
        tmux::switch_client()
    } else {
        tmux::attach()
    }
}
