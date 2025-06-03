mod cli;
mod selection_store;
mod operations;
mod modes;
mod utils;
mod config;

use clap::Parser;
use cli::{Cli, Commands};
use config::Config;

fn main() {
    let cli = Cli::parse();

    // Create the Config instance
    let config = Config::new(cli.silent, cli.verbose);

    let result = match cli.command {
        Commands::Select { items, regex, interactive, path, dry_run } => {
            // Pass the config to the handler
            modes::handle_select_command(items, regex, interactive, path, dry_run, &config)
        }
        Commands::Copy { force } => {
            // Pass the config to the handler
            operations::handle_copy_command(force, &config)
        }
        Commands::Move { force } => {
            // Pass the config to the handler
            operations::handle_move_command(force, &config)
        }
    };

    if let Err(e) = result {
        config.print_error(&e); // Use the config's error printing
        std::process::exit(1);
    }
}