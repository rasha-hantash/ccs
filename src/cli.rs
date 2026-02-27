use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccs", about = "Claude Code session manager", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Start or add a session tab
    Start {
        /// Session name
        #[arg(default_value = "session-1")]
        name: String,
        /// Working directory
        dir: Option<String>,
    },
    /// List active sessions
    #[command(alias = "ls")]
    List,
    /// Kill a single session tab
    Kill {
        /// Session name to kill
        name: String,
    },
    /// Kill all sessions
    AllKill,
    /// Reattach to existing session
    Resume,
    /// Interactive session navigator (launched by start)
    Sidebar,
    /// Handle Claude Code hook events (called by hooks, not directly)
    Hook {
        #[command(subcommand)]
        event: HookEvent,
    },
    /// Install Claude Code hooks for session status detection
    Init,
}

#[derive(Subcommand)]
pub enum HookEvent {
    /// Claude received a user prompt (UserPromptSubmit hook)
    UserPrompt,
    /// Claude finished responding (Stop hook)
    Stop,
    /// Claude is about to show an AskUserQuestion (PreToolUse hook)
    Ask,
    /// User answered an AskUserQuestion (PostToolUse hook)
    AskDone,
}
