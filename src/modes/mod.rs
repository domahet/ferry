pub mod interactive;
pub mod explicit;

use std::path::PathBuf;
use super::selection_store;
use crate::config::Config;

pub fn handle_select_command(
    items: Vec<String>,
    regex: Option<String>,
    interactive: bool,
    path: Option<String>,
    dry_run: bool,
    config: &Config,
) -> Result<(), String> {
    let resolved_start_path = path.as_deref().unwrap_or(".").to_string();
    let start_path_buf = PathBuf::from(&resolved_start_path);

    if path.is_some() && !start_path_buf.is_dir() {
        config.print_error(&format!("The specified --path '{}' is not a valid directory.", resolved_start_path));
        return Err(format!("The specified --path '{}' is not a valid directory.", resolved_start_path));
    }

    let selected_paths: Vec<PathBuf> = if interactive {
        if !items.is_empty() {
            config.print_error("Do not provide item paths directly when using --interactive. Use --path to specify a starting directory.");
            return Err("Do not provide item paths directly when using --interactive. Use --path to specify a starting directory.".to_string());
        }
        config.print_normal(&format!("Launching interactive TUI selection from {})", start_path_buf.display()));
        interactive::run_tui_selection(&start_path_buf, config)?
    } else if let Some(regex_pattern) = regex {
        if !items.is_empty() {
            config.print_error("Do not provide item paths directly when using --regex. Use --path to specify a starting directory for the search.");
            return Err("Do not provide item paths directly when using --regex. Use --path to specify a starting directory for the search.".to_string());
        }
        config.print_normal(&format!("Running REGEX selection for '{}' in {})", regex_pattern, start_path_buf.display()));
        explicit::run_regex_selection(&start_path_buf, &regex_pattern, config)?
    } else {
        if path.is_some() {
            config.print_error("The --path flag is not applicable when directly providing item paths. It is used with --regex or --interactive.");
            return Err("The --path flag is not applicable when directly providing item paths. It is used with --regex or --interactive.".to_string());
        }

        if items.is_empty() {
            config.print_normal("No arguments provided. Launching interactive TUI from current directory.");
            interactive::run_tui_selection(&PathBuf::from("."), config)? // Pass config
        } else {
            config.print_normal(&format!("Selected {} items directly (shell-expanded). (dry_run: {})", items.len(), dry_run));
            explicit::validate_and_canonicalize_items(&items, config)?
        }
    };

    if selected_paths.is_empty() {
        config.print_normal("No valid items found to select. Nothing saved.");
    } else {
        if dry_run {
            config.print_normal("Dry run: would select the following:");
            for p in &selected_paths {
                config.print_normal(&format!("  {}", p.display()));
            }
        } else {
            selection_store::write_selected_paths(&selected_paths)?;
            config.print_normal(&format!("Selected {} items and saved to selection file.", selected_paths.len()));
        }
    }
    Ok(())
}