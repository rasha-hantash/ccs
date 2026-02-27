# cove — Claude Code Session Manager

Manage multiple [Claude Code](https://docs.anthropic.com/en/docs/claude-code) sessions in tmux with a ratatui-powered sidebar navigator and real-time status detection.

## What it does

- **Multi-session tmux layout** — Each session gets a 3-pane window: Claude Code (left), interactive sidebar (top-right), and mini terminal (bottom-right).
- **Real-time status indicators** — See which sessions are working, waiting for input, or idle — powered by Claude Code hooks.
- **Interactive sidebar** — Navigate between sessions with arrow keys. Status updates live as Claude works.
- **Hook integration** — Detects `UserPromptSubmit`, `Stop`, `PreToolUse`/`PostToolUse` (for `AskUserQuestion`) events from Claude Code.

## Prerequisites

- [tmux](https://github.com/tmux/tmux) (3.2+)
- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code)

## Install

### Homebrew

```sh
brew tap rasha-hantash/cove && brew install cove-cli
```

### curl (macOS / Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/rasha-hantash/cove/releases/latest/download/cove-cli-installer.sh | sh
```

### cargo install

```sh
cargo install cove-cli
```

### From source

```sh
git clone https://github.com/rasha-hantash/cove.git
cd cove
cargo install --path .
```

## Quick Start

```sh
# Install Claude Code hooks (one-time setup)
cove init

# Start a new session
cove my-project ~/code/my-project

# Start another session
cove api-work ~/code/api

# List active sessions
cove list

# Reattach to an existing session group
cove resume

# Kill a session
cove kill my-project

# Kill all sessions
cove all-kill
```

## Commands

| Command                 | Description                                            |
| ----------------------- | ------------------------------------------------------ |
| `cove [name] [dir]`     | Start a new session (or resume if no args)             |
| `cove list` / `cove ls` | List active sessions with status and working directory |
| `cove kill <name>`      | Kill a single session                                  |
| `cove all-kill`         | Kill all sessions                                      |
| `cove resume`           | Reattach to an existing session group                  |
| `cove init`             | Install Claude Code hooks for status detection         |
| `cove sidebar`          | Launch the interactive navigator (called internally)   |
| `cove hook <event>`     | Handle hook events (called by hooks, not directly)     |

## How It Works

Cove creates a tmux session group with one window per Claude Code session. Each window has three panes:

1. **Claude pane** — runs `claude` CLI
2. **Sidebar pane** — ratatui TUI showing all sessions with live status
3. **Terminal pane** — mini shell in the session's working directory

Status detection works through Claude Code's hook system. `cove init` installs hooks into `~/.claude/settings.json` that fire `cove hook` on key events. These write JSONL event files to `~/.cove/events/`, which the sidebar reads to determine each session's state (working, waiting for input, idle).

## Configuration

Run `cove init` to install the required hooks. This adds entries to your `~/.claude/settings.json`:

- `UserPromptSubmit` — detects when Claude starts working
- `Stop` — detects when Claude finishes
- `PreToolUse` / `PostToolUse` — detects `AskUserQuestion` prompts

The hooks are non-blocking and only write small event files — they don't affect Claude Code performance.

## License

MIT
