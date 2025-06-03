
use std::path::{Path, PathBuf};
use crate::config::Config;

pub fn canonicalize_path(input_path: &Path, config: &Config) -> Result<Option<PathBuf>, String> {
    let resolved_path = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current working directory: {}", e))?;
        current_dir.join(input_path)
    };

    if !resolved_path.exists() {
        config.print_warning(&format!("Path '{}' does not exist or is inaccessible. Skipping.", resolved_path.display()));
        return Ok(None);
    }

    resolved_path.canonicalize()
        .map(Some)
        .map_err(|e| format!("Failed to canonicalize path {}: {}", resolved_path.display(), e))
}