// ── tmux Command wrappers ──

use std::process::Command;

// ── Types ──

pub struct WindowInfo {
    pub index: u32,
    pub name: String,
    pub is_active: bool,
    pub pane_path: String,
}

// ── Helpers ──

fn tmux(args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("tmux").args(args).output()
}

fn tmux_ok(args: &[&str]) -> bool {
    tmux(args).is_ok_and(|o| o.status.success())
}

fn tmux_stdout(args: &[&str]) -> Result<String, String> {
    let output = tmux(args).map_err(|e| format!("tmux: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("tmux: {}", stderr.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ── Public API ──

pub const SESSION: &str = "ccs";

pub fn has_session() -> bool {
    tmux_ok(&["has-session", "-t", SESSION])
}

pub fn list_windows() -> Result<Vec<WindowInfo>, String> {
    let out = tmux_stdout(&[
        "list-windows",
        "-t",
        SESSION,
        "-F",
        "#{window_index}|#{window_name}|#{window_active}|#{pane_current_path}",
    ])?;

    let mut windows = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        windows.push(WindowInfo {
            index: parts[0].parse().unwrap_or(0),
            name: parts[1].to_string(),
            is_active: parts[2] == "1",
            pane_path: parts[3].to_string(),
        });
    }
    Ok(windows)
}

/// List window names only (for duplicate checking).
pub fn list_window_names() -> Result<Vec<String>, String> {
    let out = tmux_stdout(&["list-windows", "-t", SESSION, "-F", "#{window_name}"])?;
    Ok(out.lines().map(|s| s.to_string()).collect())
}

pub fn is_inside_tmux() -> bool {
    std::env::var("TMUX").is_ok_and(|v| !v.is_empty())
}

pub fn new_session(name: &str, dir: &str, sidebar_bin: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args([
            "new-session",
            "-s",
            SESSION,
            "-n",
            name,
            "-c",
            dir,
            ";",
            "set-option",
            "-w",
            "remain-on-exit",
            "on",
            ";",
            "set-hook",
            "pane-died",
            "respawn-pane",
            ";",
            "split-window",
            "-v",
            "-p",
            "25",
            "-c",
            dir,
            ";",
            "split-window",
            "-t",
            ".2",
            "-h",
            "-p",
            "30",
            sidebar_bin,
            ";",
            "select-pane",
            "-t",
            ".2",
            ";",
            "respawn-pane",
            "-t",
            ".1",
            "-k",
            "claude",
        ])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux new-session failed".to_string());
    }
    Ok(())
}

pub fn new_window(name: &str, dir: &str) -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["new-window", "-t", SESSION, "-n", name, "-c", dir, "claude"])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux new-window failed".to_string());
    }
    Ok(())
}

pub fn setup_layout(name: &str, dir: &str, sidebar_bin: &str) -> Result<(), String> {
    let win = format!("{SESSION}:{name}");
    let status = Command::new("tmux")
        .args([
            "set-option",
            "-w",
            "-t",
            &win,
            "remain-on-exit",
            "on",
            ";",
            "split-window",
            "-t",
            &win,
            "-v",
            "-p",
            "25",
            "-c",
            dir,
            ";",
            "split-window",
            "-t",
            &format!("{win}.2"),
            "-h",
            "-p",
            "30",
            sidebar_bin,
            ";",
            "select-pane",
            "-t",
            &format!("{win}.2"),
        ])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux setup-layout failed".to_string());
    }
    Ok(())
}

pub fn attach() -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["attach", "-t", SESSION])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux attach failed".to_string());
    }
    Ok(())
}

pub fn switch_client() -> Result<(), String> {
    let status = Command::new("tmux")
        .args(["switch-client", "-t", SESSION])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux switch-client failed".to_string());
    }
    Ok(())
}

pub fn kill_window(name: &str) -> Result<(), String> {
    let target = format!("{SESSION}:{name}");
    tmux_stdout(&["kill-window", "-t", &target])?;
    Ok(())
}

pub fn kill_session() -> Result<(), String> {
    tmux_stdout(&["kill-session", "-t", SESSION])?;
    Ok(())
}

pub fn select_window(index: u32) -> Result<(), String> {
    let target = format!("{SESSION}:{index}");
    let status = Command::new("tmux")
        .args([
            "select-window",
            "-t",
            &target,
            ";",
            "select-pane",
            "-t",
            ":.1",
        ])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux select-window failed".to_string());
    }
    Ok(())
}

/// Info about pane .1 in each window (for state detection).
pub struct PaneInfo {
    pub window_index: u32,
    pub command: String,
    /// Unique tmux pane identifier (e.g. "%0", "%3").
    pub pane_id: String,
}

/// Get the foreground command and pane ID of pane .1 in every window.
pub fn list_pane_commands() -> Result<Vec<PaneInfo>, String> {
    let out = tmux_stdout(&[
        "list-panes",
        "-s",
        "-t",
        SESSION,
        "-F",
        "#{window_index}|#{pane_index}|#{pane_current_command}|#{pane_id}",
    ])?;

    let mut panes = Vec::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.splitn(4, '|').collect();
        if parts.len() < 4 {
            continue;
        }
        // Only pane index 1 (the Claude pane)
        if parts[1] != "1" {
            continue;
        }
        panes.push(PaneInfo {
            window_index: parts[0].parse().unwrap_or(0),
            command: parts[2].to_string(),
            pane_id: parts[3].to_string(),
        });
    }
    Ok(panes)
}

pub fn select_window_sidebar(index: u32) -> Result<(), String> {
    let target = format!("{SESSION}:{index}");
    let status = Command::new("tmux")
        .args([
            "select-window",
            "-t",
            &target,
            ";",
            "select-pane",
            "-t",
            ":.3",
        ])
        .status()
        .map_err(|e| format!("tmux: {e}"))?;

    if !status.success() {
        return Err("tmux select-window failed".to_string());
    }
    Ok(())
}
