// ── Claude Code hook handler ──
//
// Called by Claude Code hooks to write CCS state events.
// Reads JSON from stdin, determines state, appends to ~/.ccs/events/{session_id}.jsonl.
//
// Hook → state mapping:
//   UserPromptSubmit           → working
//   PreToolUse(AskUserQuestion)  → asking
//   PostToolUse(AskUserQuestion) → working
//   Stop                       → idle

use std::fs::{self, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;

use serde::Deserialize;

use crate::cli::HookEvent;

// ── Types ──

#[derive(Deserialize)]
struct HookInput {
    session_id: String,
    cwd: String,
}

// ── Helpers ──

fn events_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".ccs").join("events")
}

/// Append a state event to the session's event file.
fn write_event(session_id: &str, cwd: &str, pane_id: &str, state: &str) -> Result<(), String> {
    let dir = events_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("create events dir: {e}"))?;

    let path = dir.join(format!("{session_id}.jsonl"));
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|e| format!("open event file: {e}"))?;

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let line = format!(r#"{{"state":"{state}","cwd":"{cwd}","pane_id":"{pane_id}","ts":{ts}}}"#);
    writeln!(file, "{line}").map_err(|e| format!("write event: {e}"))?;

    Ok(())
}

// ── Public API ──

pub fn run(event: HookEvent) -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|e| format!("read stdin: {e}"))?;

    let hook: HookInput =
        serde_json::from_str(&input).map_err(|e| format!("parse hook input: {e}"))?;

    let state = match event {
        HookEvent::UserPrompt | HookEvent::AskDone => "working",
        HookEvent::Stop => "idle",
        HookEvent::Ask => "asking",
    };

    // $TMUX_PANE uniquely identifies which tmux pane Claude is running in.
    // This lets the sidebar distinguish sessions even when they share a cwd.
    let pane_id = std::env::var("TMUX_PANE").unwrap_or_default();

    write_event(&hook.session_id, &hook.cwd, &pane_id, state)
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_write_event_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let events = dir.path().join("events");

        fs::create_dir_all(&events).unwrap();
        let path = events.join("test-session.jsonl");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(file, r#"{{"state":"working","cwd":"/tmp","ts":1234}}"#).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains(r#""state":"working""#));
        assert!(content.contains(r#""cwd":"/tmp""#));
    }
}
