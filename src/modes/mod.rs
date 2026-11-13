pub mod interactive;
pub mod explicit;

use std::path::PathBuf;
use super::selection_store;
use crate::config::Config;
use crate::operations::plural;

pub fn handle_select_command(
    items: Vec<PathBuf>,
    regex: Option<String>,
    interactive: bool,
    path: Option<PathBuf>,
    dry_run: bool,
    config: &Config,
) -> Result<(), String> {
    let start_path = path.clone().unwrap_or_else(|| PathBuf::from("."));
    if !start_path.is_dir() {
        return Err(format!("The specified --path '{}' is not a valid directory.", start_path.display()));
    }

    let selected_paths: Vec<PathBuf> = if let Some(regex_pattern) = regex {
        config.print_verbose(&format!("Selecting files matching '{}' in {}", regex_pattern, start_path.display()));
        explicit::run_regex_selection(&start_path, &regex_pattern, config)?
    } else if interactive || items.is_empty() {
        match interactive::run_tui_selection(&start_path, config)? {
            Some(paths) => paths,
            None => {
                config.print_normal("Cancelled. Selection unchanged.");
                return Ok(());
            }
        }
    } else {
        explicit::validate_and_canonicalize_items(&items, config)?
    };

    if selected_paths.is_empty() {
        config.print_normal("No valid items found to select. Nothing saved.");
    } else if dry_run {
        config.print_normal("Dry run: would select the following:");
        for p in &selected_paths {
            config.print_normal(&format!("  {}", p.display()));
        }
    } else {
        let total = selection_store::write_selected_paths(&selected_paths)?;
        config.print_normal(&format!(
            "Selected {} ({} in selection).",
            plural(selected_paths.len(), "item"),
            total
        ));
    }
    Ok(())
}
