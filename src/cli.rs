// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author = "domahet", version, about = "A ferry for your files", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Suppress all output to stdout. Only errors will be printed to stderr.
    #[arg(long, default_value_t = false, conflicts_with = "verbose")]
    pub silent: bool,

    /// Print all available information, including file names during selection.
    #[arg(long, default_value_t = false, conflicts_with = "silent")]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Select files for copying or moving
    Select {
        /// Paths to items to select directly (shell-globbed).
        /// These are positional arguments. They are ignored if --regex or --interactive is used.
        items: Vec<String>, 

        /// Select items by regular expression (e.g., "^doc_\\d{3}\\.pdf$")
        /// This activates regex search mode.
        #[arg(long, conflicts_with = "interactive")]
        regex: Option<String>, 

        /// Launch an interactive Text User Interface (TUI) for selection.
        /// This activates TUI mode.
        #[arg(short = 'i', long)]
        interactive: bool, 

        /// Specify the starting directory for the regex search or interactive TUI.
        /// Defaults to the current working directory.
        #[arg(short = 'P', long)]
        path: Option<String>, 

        /// Perform a dry run: show what would be selected without saving to the selection file.
        #[arg(long)]
        dry_run: bool, 
    },
    /// Copy previously selected items to the current directory
    Copy {
        /// Overwrite existing files without prompting.
        #[arg(short = 'f', long)]
        force: bool, 
    },
    /// Move previously selected items to the current directory
    Move {
        /// Overwrite existing files without prompting.
        #[arg(short = 'f', long)]
        force: bool, 
    },
}