use std::path::{Path, PathBuf};
use regex::Regex;
use walkdir::WalkDir;
use crate::utils::fs_helpers;

pub fn run_regex_selection(start_path: &Path, pattern_str: &str) -> Result<Vec<PathBuf>, String> {
    let regex = Regex::new(pattern_str)
        .map_err(|e| format!("Invalid regex pattern: {}", e))?;

    let mut selected_paths = Vec::new();

    for entry_result in WalkDir::new(start_path) {
        let entry = entry_result
            .map_err(|e| format!("Error traversing directory {}: {}", start_path.display(), e))?;
        let path = entry.path();

        if regex.is_match(path.to_str().unwrap_or("")) {
            if let Some(abs_path) = fs_helpers::canonicalize_path(path)? {
                selected_paths.push(abs_path);
            }
        }
    }
    Ok(selected_paths)
}

pub fn validate_and_canonicalize_items(item_strs: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut selected_paths = Vec::new();
    for p_str in item_strs {
        let path = PathBuf::from(p_str);
        if let Some(abs_path) = fs_helpers::canonicalize_path(&path)? {
            selected_paths.push(abs_path);
        }
    }
    Ok(selected_paths)
}