use std::path::PathBuf;
use std::fs;
use directories::ProjectDirs;

const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "ferry-cli";
const APP_NAME: &str = "ferry";
const SELECTION_FILE_NAME: &str = "selection";

fn get_ferry_data_dir() -> Result<PathBuf, String> {
    if let Some(proj_dirs) = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME) {
        let cache_dir = proj_dirs.cache_dir().to_path_buf();
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory {}: {}", cache_dir.display(), e))?;
        Ok(cache_dir)
    } else {
        Err("Could not determine a suitable cache directory for Ferry.".to_string())
    }
}

pub fn get_selection_file_path() -> Result<PathBuf, String> {
    let mut path = get_ferry_data_dir()?;
    path.push(SELECTION_FILE_NAME);
    Ok(path)
}

pub fn write_selected_paths(paths: &[PathBuf]) -> Result<(), String> {
    let file_path = get_selection_file_path()?;
    let content: Vec<String> = paths.iter().filter_map(|p| p.to_str().map(|s| s.to_string())).collect();
    fs::write(&file_path, content.join("\n"))
        .map_err(|e| format!("Failed to write selection to {}: {}", file_path.display(), e))
}

pub fn read_selected_paths() -> Result<Vec<PathBuf>, String> {
    let file_path = get_selection_file_path()?;
    if !file_path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read selection from {}: {}", file_path.display(), e))?;
    Ok(content.lines().map(PathBuf::from).collect())
}

pub fn clear_selection_file() -> Result<(), String> {
    let file_path = get_selection_file_path()?;
    if file_path.exists() {
        fs::remove_file(&file_path)
            .map_err(|e| format!("Failed to clear selection file {}: {}", file_path.display(), e))?;
    }
    Ok(())
}