// ── Hook installation for Claude Code ──
//
// Adds Cove hook entries to ~/.claude/settings.json so Claude Code
// calls `cove hook user-prompt` and `cove hook stop` on session events.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

// ── Helpers ──

fn settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".claude").join("settings.json")
}

fn cove_bin_path() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(canonical) = fs::canonicalize(exe) {
            return canonical.to_string_lossy().to_string();
        }
    }
    let home = std::env::var("HOME").unwrap_or_default();
    format!("{home}/.local/bin/cove")
}

/// Check if Cove hooks are already installed in settings.json with the correct binary path.
/// Returns false if hooks are missing OR if the binary path is stale.
pub fn hooks_installed(path: &Path) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Must have the ask hook (detects old installs missing PreToolUse)
    // AND point to the current binary (detects stale paths after rename/move)
    let bin = cove_bin_path();
    let ask_cmd = format!("{bin} hook ask");
    content.contains(&ask_cmd)
}

/// Install Cove hooks into settings.json.
/// Appends to existing hook arrays — does not overwrite.
pub fn install_hooks(path: &Path) -> Result<(), String> {
    install_hooks_with_bin(path, &cove_bin_path())
}

/// Check if a hook array already contains an entry whose command includes `needle`.
fn has_hook_command(arr: &[Value], needle: &str) -> bool {
    arr.iter().any(|entry| {
        entry["hooks"]
            .as_array()
            .map(|hooks| {
                hooks.iter().any(|h| {
                    h["command"]
                        .as_str()
                        .map(|c| c.contains(needle))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
    })
}

/// Remove any hook entries whose command contains `needle` from an array.
/// Returns the number of entries removed.
fn remove_hook_commands(arr: &mut Vec<Value>, needle: &str) -> usize {
    let before = arr.len();
    arr.retain(|entry| {
        let is_cove = entry["hooks"]
            .as_array()
            .map(|hooks| {
                hooks.iter().any(|h| {
                    h["command"]
                        .as_str()
                        .map(|c| c.contains(needle))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        !is_cove
    });
    before - arr.len()
}

/// Check if settings.json has cove hooks pointing to a different binary path.
pub fn has_stale_hooks(path: &Path, current_bin: &str) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    // Has some cove hook commands but NOT with the current binary path
    content.contains(" hook user-prompt") && !content.contains(current_bin)
}

fn install_hooks_with_bin(path: &Path, bin: &str) -> Result<(), String> {
    let mut settings: Value = if path.exists() {
        let content = fs::read_to_string(path).map_err(|e| format!("read settings: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("parse settings: {e}"))?
    } else {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create settings dir: {e}"))?;
        }
        serde_json::json!({})
    };

    let hooks = settings
        .as_object_mut()
        .ok_or("settings.json is not an object")?
        .entry("hooks")
        .or_insert_with(|| serde_json::json!({}));

    let hooks_obj = hooks.as_object_mut().ok_or("hooks is not an object")?;

    // Each entry: (hook_type, matcher, cove_command)
    let entries: &[(&str, &str, &str)] = &[
        ("UserPromptSubmit", "*", "hook user-prompt"),
        ("Stop", "*", "hook stop"),
        ("PreToolUse", "AskUserQuestion", "hook ask"),
        ("PostToolUse", "AskUserQuestion", "hook ask-done"),
    ];

    for &(hook_type, matcher, cmd) in entries {
        let arr = hooks_obj
            .entry(hook_type)
            .or_insert_with(|| serde_json::json!([]));
        let arr = arr
            .as_array_mut()
            .ok_or(format!("{hook_type} is not an array"))?;

        // Remove stale cove hooks (different binary path) before adding new ones
        remove_hook_commands(arr, "cove hook");

        let full_cmd = format!("{bin} {cmd}");
        if !has_hook_command(arr, &full_cmd) {
            arr.push(serde_json::json!({
                "matcher": matcher,
                "hooks": [{
                    "type": "command",
                    "command": full_cmd,
                    "async": true,
                    "timeout": 5
                }]
            }));
        }
    }

    let output =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("serialize settings: {e}"))?;
    fs::write(path, output).map_err(|e| format!("write settings: {e}"))?;

    Ok(())
}

// ── Public API ──

pub fn run() -> Result<(), String> {
    let path = settings_path();

    if hooks_installed(&path) {
        println!("Cove hooks are already installed in ~/.claude/settings.json");
        return Ok(());
    }

    let bin = cove_bin_path();
    let stale = has_stale_hooks(&path, &bin);

    install_hooks(&path)?;

    if stale {
        println!("Updated Cove hooks in ~/.claude/settings.json");
        println!("  (old binary path was replaced with {bin})");
    } else {
        println!("Installed Cove hooks in ~/.claude/settings.json");
    }
    println!("  UserPromptSubmit              → cove hook user-prompt");
    println!("  Stop                          → cove hook stop");
    println!("  PreToolUse(AskUserQuestion)   → cove hook ask");
    println!("  PostToolUse(AskUserQuestion)  → cove hook ask-done");

    Ok(())
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hooks_installed_no_file() {
        assert!(!hooks_installed(Path::new("/nonexistent/settings.json")));
    }

    #[test]
    fn test_hooks_installed_empty() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{}").unwrap();

        assert!(!hooks_installed(&path));
    }

    #[test]
    fn test_hooks_installed_only_old_hooks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        // Old installation — has "cove hook stop" but not "cove hook ask"
        fs::write(
            &path,
            r#"{"hooks":{"Stop":[{"hooks":[{"command":"cove hook stop"}]}]}}"#,
        )
        .unwrap();

        assert!(!hooks_installed(&path));
    }

    #[test]
    fn test_hooks_installed_present() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{}").unwrap();

        // Install hooks with the actual binary path, then verify detection
        install_hooks(&path).unwrap();
        assert!(hooks_installed(&path));
    }

    #[test]
    fn test_install_hooks_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{}").unwrap();

        install_hooks_with_bin(&path, "cove").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("cove hook user-prompt"));
        assert!(content.contains("cove hook stop"));
        assert!(content.contains("cove hook ask\""));
        assert!(content.contains("cove hook ask-done"));

        let parsed: Value = serde_json::from_str(&content).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();
        assert_eq!(hooks["UserPromptSubmit"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["PostToolUse"].as_array().unwrap().len(), 1);

        // PreToolUse should use AskUserQuestion matcher
        let pre = &hooks["PreToolUse"].as_array().unwrap()[0];
        assert_eq!(pre["matcher"].as_str().unwrap(), "AskUserQuestion");
    }

    #[test]
    fn test_install_hooks_preserves_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(
            &path,
            r#"{"hooks":{"Stop":[{"matcher":"*","hooks":[{"type":"command","command":"afplay sound.aiff"}]}]}}"#,
        )
        .unwrap();

        install_hooks_with_bin(&path, "cove").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();

        // Stop should have 2 entries: original + Cove
        let stop = parsed["hooks"]["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2);
        assert!(
            stop[0]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains("afplay")
        );
        assert!(
            stop[1]["hooks"][0]["command"]
                .as_str()
                .unwrap()
                .contains("cove hook stop")
        );
    }

    #[test]
    fn test_install_hooks_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        fs::write(&path, "{}").unwrap();

        install_hooks_with_bin(&path, "cove").unwrap();
        install_hooks_with_bin(&path, "cove").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();

        // Each hook type should still have exactly 1 Cove entry
        assert_eq!(hooks["UserPromptSubmit"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["PostToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_install_hooks_creates_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("subdir").join("settings.json");

        install_hooks_with_bin(&path, "cove").unwrap();

        assert!(path.exists());
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("cove hook ask"));
    }

    #[test]
    fn test_install_hooks_upgrades_old_install() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        // Simulate old installation with only UserPromptSubmit + Stop
        fs::write(
            &path,
            r#"{"hooks":{"UserPromptSubmit":[{"matcher":"*","hooks":[{"type":"command","command":"cove hook user-prompt","async":true,"timeout":5}]}],"Stop":[{"matcher":"*","hooks":[{"type":"command","command":"cove hook stop","async":true,"timeout":5}]}]}}"#,
        )
        .unwrap();

        install_hooks_with_bin(&path, "cove").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();

        // Old hooks should not be duplicated
        assert_eq!(hooks["UserPromptSubmit"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["Stop"].as_array().unwrap().len(), 1);
        // New hooks should be added
        assert_eq!(hooks["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(hooks["PostToolUse"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_install_hooks_replaces_stale_binary_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        // Hooks with old binary path + a non-cove hook that should be preserved
        fs::write(
            &path,
            r#"{"hooks":{"Stop":[{"matcher":"*","hooks":[{"type":"command","command":"afplay sound.aiff"}]},{"matcher":"*","hooks":[{"type":"command","command":"/old/path/cove hook stop","async":true,"timeout":5}]}],"UserPromptSubmit":[{"matcher":"*","hooks":[{"type":"command","command":"/old/path/cove hook user-prompt","async":true,"timeout":5}]}]}}"#,
        )
        .unwrap();

        assert!(has_stale_hooks(&path, "/new/path/cove"));

        install_hooks_with_bin(&path, "/new/path/cove").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&content).unwrap();
        let hooks = parsed["hooks"].as_object().unwrap();

        // Stop should have 2 entries: preserved afplay + new cove
        let stop = hooks["Stop"].as_array().unwrap();
        assert_eq!(stop.len(), 2);
        assert!(stop[0]["hooks"][0]["command"].as_str().unwrap().contains("afplay"));
        assert!(stop[1]["hooks"][0]["command"].as_str().unwrap().contains("/new/path/cove hook stop"));

        // Old path should be gone
        assert!(!content.contains("/old/path/cove"));

        // UserPromptSubmit should have exactly 1 (replaced)
        assert_eq!(hooks["UserPromptSubmit"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_hooks_installed_stale_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        // Has cove hooks but with a different binary path
        fs::write(
            &path,
            r#"{"hooks":{"UserPromptSubmit":[{"hooks":[{"command":"/old/path/cove hook user-prompt"}]}],"PreToolUse":[{"hooks":[{"command":"/old/path/cove hook ask"}]}]}}"#,
        )
        .unwrap();

        // hooks_installed should return false because binary path doesn't match
        assert!(!hooks_installed(&path));
        assert!(has_stale_hooks(&path, &cove_bin_path()));
    }
}
