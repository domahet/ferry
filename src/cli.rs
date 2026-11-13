use clap::{Parser, Subcommand};
use std::path::PathBuf;

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
    /// Add files or directories to the selection for copying or moving.
    /// With no items or --regex, opens the interactive picker.
    Select {
        /// Files or directories to select (shell globs are expanded by your shell).
        #[arg(conflicts_with_all = ["regex", "interactive", "path"])]
        items: Vec<PathBuf>,

        /// Select files by regular expression, searching --path recursively.
        /// The pattern is matched against each file's path relative to --path,
        /// using '/' as separator (e.g. "^doc_\d{3}\.pdf$" or "^src/.*\.rs$").
        #[arg(long, conflicts_with = "interactive")]
        regex: Option<String>,

        /// Pick items in an interactive terminal UI (the default with no items).
        #[arg(short = 'i', long)]
        interactive: bool,

        /// Directory to search with --regex or browse in the picker.
        /// Defaults to the current directory.
        #[arg(short = 'P', long)]
        path: Option<PathBuf>,

        /// Show what would be selected without changing the selection.
        #[arg(long)]
        dry_run: bool,
    },
    /// Copy the selected items into the current directory and clear the selection
    Copy {
        /// Replace items that already exist in the current directory.
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// Move the selected items into the current directory and clear the selection
    Move {
        /// Replace items that already exist in the current directory.
        #[arg(short = 'f', long)]
        force: bool,
    },
    /// List the selected items
    List {
        /// Display absolute paths. This is the default.
        #[arg(long, default_value_t = false, conflicts_with = "relative", hide = true)]
        absolute: bool,

        /// Display paths relative to the current working directory.
        #[arg(long, default_value_t = false, conflicts_with = "absolute")]
        relative: bool,
    },
    /// Remove items from the selection without copying or moving them
    Deselect {
        /// Paths of selected items to remove. They do not need to exist anymore.
        #[arg(required_unless_present = "missing")]
        items: Vec<PathBuf>,

        /// Remove every selected item that no longer exists.
        #[arg(long)]
        missing: bool,
    },
    /// Clear the whole selection
    Clear,
}