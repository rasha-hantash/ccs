// ── Event loop for sidebar ──

use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};

// ── Types ──

pub enum Action {
    Up,
    Down,
    Select,
    Quit,
    Tick,
}

// ── Public API ──

/// Poll for input events with a 100ms timeout. Returns accumulated actions.
/// Batches rapid arrow presses into single moves (key draining).
pub fn poll() -> Vec<Action> {
    let mut actions = Vec::new();

    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        // Process first event
        if let Ok(Event::Key(key)) = event::read()
            && let Some(action) = key_to_action(key)
        {
            actions.push(action);
        }

        // Drain queued keys (batch rapid arrow presses)
        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read()
                && let Some(action) = key_to_action(key)
            {
                actions.push(action);
            }
        }
    }

    if actions.is_empty() {
        actions.push(Action::Tick);
    }

    actions
}

// ── Helpers ──

fn key_to_action(key: KeyEvent) -> Option<Action> {
    // Only handle key press events (ignore release/repeat)
    if key.kind != crossterm::event::KeyEventKind::Press {
        return None;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Enter => Some(Action::Select),
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Some(Action::Quit),
        _ => None,
    }
}
