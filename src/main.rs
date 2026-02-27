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
        Command::Start { name, dir } => commands::start::run(&name, dir.as_deref()),
        Command::List => commands::list::run(),
        Command::Kill { name } => commands::kill::run(&name),
        Command::AllKill => commands::kill::run_all(),
        Command::Resume => commands::resume::run(),
        Command::Sidebar => sidebar::app::run(),
        Command::Hook { event } => commands::hook::run(event),
        Command::Init => commands::init::run(),
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
