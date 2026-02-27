use crate::colors::*;
use crate::tmux;

pub fn run(name: &str) -> Result<(), String> {
    if !tmux::has_session() {
        println!("{ANSI_OVERLAY}No active ccs session.{ANSI_RESET}");
        return Err(String::new());
    }

    tmux::kill_window(name)?;
    println!("Killed: {ANSI_PEACH}{name}{ANSI_RESET}");
    Ok(())
}

pub fn run_all() -> Result<(), String> {
    if !tmux::has_session() {
        println!("{ANSI_OVERLAY}No active ccs session.{ANSI_RESET}");
        return Err(String::new());
    }

    tmux::kill_session()?;
    println!("Killed all sessions.");
    Ok(())
}
