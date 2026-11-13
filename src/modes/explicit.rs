use std::path::{Path, PathBuf};
use regex::Regex;
use walkdir::WalkDir;
use crate::utils::fs_helpers;
use crate::config::Config;

pub fn run_regex_selection(start_path: &Path, pattern_str: &str, config: &Config) -> Result<Vec<PathBuf>, String> { 
    let regex = Regex::new(pattern_str)
        .map_err(|e| format!("Invalid regex pattern: {}", e))?;

    let mut selected_paths = Vec::new();

    for entry_result in WalkDir::new(start_path).sort_by_file_name() {
        let entry = entry_result
            .map_err(|e| format!("Error traversing directory {}: {}", start_path.display(), e))?;
        if entry.file_type().is_dir() {
            continue;
        }
        let path = entry.path();

        // Match against the path relative to the search root, with '/' as the
        // separator, so '^name$' anchors work and patterns are portable.
        let relative = path.strip_prefix(start_path).unwrap_or(path);
        let candidate = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        if regex.is_match(&candidate) {
            if let Some(abs_path) = fs_helpers::canonicalize_path(path, config)? {
                config.print_verbose(&format!("Selected by regex: {}", abs_path.display()));
                selected_paths.push(abs_path);
            }
        }
    }
    Ok(selected_paths)
}

pub fn validate_and_canonicalize_items(items: &[PathBuf], config: &Config) -> Result<Vec<PathBuf>, String> {
    let mut selected_paths = Vec::new();
    for path in items {
        config.print_verbose(&format!("Attempting to canonicalize: '{}'", path.display()));
        if let Some(abs_path) = fs_helpers::canonicalize_path(path, config)? {
            config.print_verbose(&format!("Successfully canonicalized to: '{}'", abs_path.display()));
            selected_paths.push(abs_path);
        }
    }
    Ok(selected_paths)
}