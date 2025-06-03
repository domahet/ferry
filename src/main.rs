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

    let config = Config::new(cli.silent, cli.verbose);

    let result = match cli.command {
        Commands::Select { items, regex, interactive, path, dry_run } => {
            modes::handle_select_command(items, regex, interactive, path, dry_run, &config)
        }
        Commands::Copy { force } => {
            operations::handle_copy_command(force, &config)
        }
        Commands::Move { force } => {
            operations::handle_move_command(force, &config)
        }
        Commands::List { absolute, relative } => {
            operations::handle_list_command(absolute, relative, &config)
        }
    };

    if let Err(e) = result {
        config.print_error(&e);
        std::process::exit(1);
    }
}