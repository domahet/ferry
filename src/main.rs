mod cli;
mod selection_store;
mod operations;
mod modes;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Select { items, regex, interactive, path, dry_run } => {
            modes::handle_select_command(items, regex, interactive, path, dry_run)
        }
        Commands::Copy { force } => {
            operations::handle_copy_command(force)
        }
        Commands::Move { force } => {
            operations::handle_move_command(force)
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}