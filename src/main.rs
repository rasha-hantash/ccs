mod cli;
mod colors;
mod commands;
mod sidebar;
mod tmux;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Command::List) => commands::list::run(),
        Some(Command::Kill { name }) => commands::kill::run(&name),
        Some(Command::AllKill) => commands::kill::run_all(),
        Some(Command::Resume) => commands::resume::run(),
        Some(Command::Sidebar) => sidebar::app::run(),
        Some(Command::Hook { event }) => commands::hook::run(event),
        Some(Command::Init) => commands::init::run(),
        None => {
            // Default behavior: start a session or resume
            match cli.name {
                Some(name) => commands::start::run(&name, cli.dir.as_deref()),
                None => {
                    if tmux::has_session() {
                        commands::resume::run()
                    } else {
                        commands::start::run("session", Some("."))
                    }
                }
            }
        }
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
