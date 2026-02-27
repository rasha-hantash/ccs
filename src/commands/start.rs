use std::io::{self, Write};
use std::path::PathBuf;

use crate::colors::*;
use crate::commands::init;
use crate::tmux;

// ── Helpers ──

fn resolve_sidebar_bin() -> String {
    // Try to find the binary we're running from (works after `cargo install` or symlink)
    if let Ok(exe) = std::env::current_exe()
        && let Ok(canonical) = std::fs::canonicalize(exe)
    {
        return canonical.to_string_lossy().to_string();
    }
    // Fallback to the expected install location
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.local/bin/ccs")
}

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".claude").join("settings.json")
}

/// Prompt user to install hooks if not already present.
fn check_hooks() {
    let path = settings_path();
    if init::hooks_installed(&path) {
        return;
    }

    println!(
        "CCS needs Claude Code hooks to show session status (Working/Idle/Asking).\n\
         This adds two async hooks to ~/.claude/settings.json:\n\
         {ANSI_PEACH}  UserPromptSubmit{ANSI_RESET}  detects when you send a message\n\
         {ANSI_PEACH}  Stop{ANSI_RESET}              detects when Claude finishes responding\n"
    );
    print!("Add CCS hooks? [Y/n] ");
    let _ = io::stdout().flush();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return;
    }

    let answer = input.trim().to_lowercase();
    if answer.is_empty() || answer == "y" || answer == "yes" {
        match init::install_hooks(&path) {
            Ok(()) => println!("Hooks installed.\n"),
            Err(e) => eprintln!("Failed to install hooks: {e}\n"),
        }
    } else {
        println!("Skipped. Run `ccs init` later to enable status indicators.\n");
    }
}

// ── Public API ──

pub fn run(name: &str, dir: Option<&str>) -> Result<(), String> {
    let dir = dir.unwrap_or(".");
    let dir = std::fs::canonicalize(dir)
        .map_err(|e| format!("invalid directory '{dir}': {e}"))?
        .to_string_lossy()
        .to_string();

    let sidebar_bin = resolve_sidebar_bin();
    let sidebar_cmd = format!("{sidebar_bin} sidebar");

    // First-run: prompt to install hooks if needed
    check_hooks();

    if tmux::has_session() {
        // Reject duplicate window names
        let names = tmux::list_window_names()?;
        if names.iter().any(|n| n == name) {
            return Err(format!(
                "Session '{ANSI_PEACH}{name}{ANSI_RESET}' already exists. Pick a different name."
            ));
        }

        tmux::new_window(name, &dir)?;
        tmux::setup_layout(name, &dir, &sidebar_cmd)?;

        // If outside tmux, attach so the user sees it
        if !tmux::is_inside_tmux() {
            tmux::attach()?;
        }
    } else {
        // No session — create from scratch. Must run outside tmux for proper dimensions.
        if tmux::is_inside_tmux() {
            return Err(format!(
                "No ccs session exists. Run from outside tmux first:\n  \
                 {ANSI_PEACH}ccs start{ANSI_RESET} {name} {dir}"
            ));
        }

        tmux::new_session(name, &dir, &sidebar_cmd)?;
    }

    Ok(())
}
