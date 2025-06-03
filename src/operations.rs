use super::selection_store;
use std::fs;

pub fn handle_copy_command(force: bool) -> Result<(), String> {
    let paths = selection_store::read_selected_paths()?;
    if paths.is_empty() {
        println!("No items selected. Run 'ferry select' first.");
        return Ok(());
    }

    println!("Copying {} selected items", paths.len());

    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    for source_path in &paths {
        let file_name = source_path.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source_path.display()))?;

        let destination_path = current_dir.join(file_name);

        if destination_path.exists() {
            if !force {
                return Err(format!(
                    "Destination file '{}' already exists. Use --force to overwrite.",
                    destination_path.display()
                ));
            } else {
                println!("Overwriting existing file: {}", destination_path.display());
            }
        }

        fs::copy(source_path, &destination_path)
            .map_err(|e| format!(
                "Failed to copy '{}' to '{}': {}",
                source_path.display(),
                destination_path.display(),
                e
            ))?;

        println!("Copied '{}' to '{}'", source_path.display(), destination_path.display());
    }

    selection_store::clear_selection_file()?;
    println!("Copy complete. Selection cleared.");
    Ok(())
}

pub fn handle_move_command(force: bool) -> Result<(), String> {
    let paths = selection_store::read_selected_paths()?;
    if paths.is_empty() {
        println!("No items selected. Run 'ferry select' first.");
        return Ok(());
    }

    println!("Moving {} selected items", paths.len());

    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    for source_path in &paths {
        let file_name = source_path.file_name()
            .ok_or_else(|| format!("Invalid source path: {}", source_path.display()))?;

        let destination_path = current_dir.join(file_name);

        if destination_path.exists() {
            if !force {
                return Err(format!(
                    "Destination file '{}' already exists. Use --force to overwrite.",
                    destination_path.display()
                ));
            } else {
                println!("Overwriting existing file: {}", destination_path.display());
                if destination_path.is_file() {
                    fs::remove_file(&destination_path)
                        .map_err(|e| format!("Failed to remove existing file '{}' before move: {}", destination_path.display(), e))?;
                } else if destination_path.is_dir() {
                    fs::remove_dir_all(&destination_path)
                        .map_err(|e| format!("Failed to remove existing directory '{}' before move: {}", destination_path.display(), e))?;
                }
            }
        }

        fs::rename(source_path, &destination_path)
            .map_err(|e| format!(
                "Failed to move '{}' to '{}': {}",
                source_path.display(),
                destination_path.display(),
                e
            ))?;

        println!("Moved '{}' to '{}'", source_path.display(), destination_path.display());
    }

    selection_store::clear_selection_file()?;
    println!("Move complete. Selection cleared.");
    Ok(())
}