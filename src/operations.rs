use super::selection_store;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::config::Config;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Copy,
    Move,
}

impl Mode {
    fn verb(self) -> &'static str {
        match self {
            Mode::Copy => "copy",
            Mode::Move => "move",
        }
    }

    fn gerund(self) -> &'static str {
        match self {
            Mode::Copy => "Copying",
            Mode::Move => "Moving",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Mode::Copy => "Copy",
            Mode::Move => "Move",
        }
    }

    fn past(self) -> &'static str {
        match self {
            Mode::Copy => "Copied",
            Mode::Move => "Moved",
        }
    }
}

pub fn handle_copy_command(force: bool, config: &Config) -> Result<(), String> {
    transfer(Mode::Copy, force, config)
}

pub fn handle_move_command(force: bool, config: &Config) -> Result<(), String> {
    transfer(Mode::Move, force, config)
}

pub fn plural(n: usize, word: &str) -> String {
    if n == 1 { format!("{} {}", n, word) } else { format!("{} {}s", n, word) }
}

fn transfer(mode: Mode, force: bool, config: &Config) -> Result<(), String> {
    let paths = selection_store::read_selected_paths()?;
    if paths.is_empty() {
        config.print_normal("No items selected. Run 'ferry select' first.");
        return Ok(());
    }

    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let current_dir = current_dir.canonicalize().unwrap_or(current_dir);

    // Validate everything up front so a problem with one item never leaves
    // the transfer half done.
    let plan = plan_transfer(&paths, &current_dir, force)?;

    config.print_normal(&format!("{} {}", mode.gerund(), plural(plan.len(), "selected item")));

    let mut remaining: Vec<PathBuf> = paths.clone();
    for (source_path, destination_path) in &plan {
        if destination_path.symlink_metadata().is_ok() {
            config.print_normal(&format!("Overwriting existing item: {}", destination_path.display()));
        }

        if let Err(e) = place(mode, source_path, destination_path, config) {
            // Keep only what has not been transferred, so a retry picks up where this left off.
            selection_store::replace_selected_paths(&remaining)?;
            return Err(format!(
                "Failed to {} '{}' to '{}': {}\nItems not yet transferred remain selected.",
                mode.verb(),
                source_path.display(),
                destination_path.display(),
                e
            ));
        }

        remaining.retain(|p| p != source_path);
        selection_store::replace_selected_paths(&remaining)?;
        config.print_normal(&format!("{} '{}' to '{}'", mode.past(), source_path.display(), destination_path.display()));
    }

    config.print_normal(&format!("{} complete. Selection cleared.", mode.noun()));
    Ok(())
}

/// Pairs each selected source with its destination, refusing anything that
/// would fail midway or destroy data.
fn plan_transfer(paths: &[PathBuf], current_dir: &Path, force: bool) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut problems = Vec::new();
    let mut conflicts = Vec::new();
    let mut by_name: HashMap<&std::ffi::OsStr, &PathBuf> = HashMap::new();
    let mut plan = Vec::new();

    for source_path in paths {
        let Some(file_name) = source_path.file_name() else {
            problems.push(format!("'{}' has no file name", source_path.display()));
            continue;
        };
        if source_path.symlink_metadata().is_err() {
            problems.push(format!("'{}' no longer exists", source_path.display()));
            continue;
        }
        if let Some(other) = by_name.insert(file_name, source_path) {
            problems.push(format!(
                "'{}' and '{}' would both be placed at '{}'",
                other.display(),
                source_path.display(),
                current_dir.join(file_name).display()
            ));
            continue;
        }

        let destination_path = current_dir.join(file_name);
        if destination_path == *source_path {
            problems.push(format!("'{}' is already in the current directory", source_path.display()));
            continue;
        }
        if current_dir.starts_with(source_path) {
            problems.push(format!(
                "cannot place '{}' inside itself (current directory is within it)",
                source_path.display()
            ));
            continue;
        }
        if source_path.starts_with(&destination_path) {
            problems.push(format!(
                "'{}' is inside '{}', which it would replace",
                source_path.display(),
                destination_path.display()
            ));
            continue;
        }
        if destination_path.symlink_metadata().is_ok() && !force {
            conflicts.push(format!("'{}'", destination_path.display()));
        }
        plan.push((source_path.clone(), destination_path));
    }

    for (i, (a, _)) in plan.iter().enumerate() {
        for (b, _) in &plan[i + 1..] {
            if a.starts_with(b) || b.starts_with(a) {
                problems.push(format!(
                    "'{}' and '{}' are both selected but one contains the other",
                    a.display(),
                    b.display()
                ));
            }
        }
    }

    if !problems.is_empty() {
        return Err(format!(
            "Nothing was transferred:\n  {}\nAdjust the selection ('ferry list', 'ferry deselect') and try again.",
            problems.join("\n  ")
        ));
    }
    if !conflicts.is_empty() {
        return Err(format!(
            "Nothing was transferred. These destinations already exist (use --force to overwrite):\n  {}",
            conflicts.join("\n  ")
        ));
    }
    Ok(plan)
}

/// Copies or moves `source` to `destination`. An existing destination is set
/// aside first and only deleted once the new item is in place, so a failure
/// never loses it.
fn place(mode: Mode, source: &Path, destination: &Path, config: &Config) -> io::Result<()> {
    let backup = if destination.symlink_metadata().is_ok() {
        let backup = backup_path(destination);
        fs::rename(destination, &backup)?;
        Some(backup)
    } else {
        None
    };

    let result = match mode {
        Mode::Copy => copy_item(source, destination),
        Mode::Move => move_item(source, destination, config),
    };

    match (result, backup) {
        (Ok(()), Some(backup)) => {
            // The transfer itself succeeded; a leftover backup is only worth a warning.
            if let Err(e) = remove_item(&backup) {
                config.print_warning(&format!(
                    "Could not remove the replaced item, left at '{}': {}",
                    backup.display(),
                    e
                ));
            }
            Ok(())
        }
        (Ok(()), None) => Ok(()),
        (Err(e), backup) => {
            // Clean up any partial result, then put the original back.
            if destination.symlink_metadata().is_ok() && source.symlink_metadata().is_ok() {
                let _ = remove_item(destination);
            }
            if let Some(backup) = backup {
                let _ = fs::rename(&backup, destination);
            }
            Err(e)
        }
    }
}

fn backup_path(destination: &Path) -> PathBuf {
    let name = destination.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let parent = destination.parent().unwrap_or(Path::new("."));
    let mut n = 0u32;
    loop {
        let candidate = parent.join(format!(".{}.ferry-old-{}-{}", name, std::process::id(), n));
        if candidate.symlink_metadata().is_err() {
            return candidate;
        }
        n += 1;
    }
}

/// Copies a file, or a directory and everything in it. Symlinks inside a
/// directory are recreated as symlinks rather than followed.
fn copy_item(source: &Path, destination: &Path) -> io::Result<()> {
    if !fs::metadata(source)?.is_dir() {
        return fs::copy(source, destination).map(|_| ());
    }

    // Directory permissions are applied last so a read-only directory can still be filled.
    let mut directory_permissions = Vec::new();
    for entry in WalkDir::new(source) {
        let entry = entry.map_err(io::Error::other)?;
        let relative = entry.path().strip_prefix(source).map_err(io::Error::other)?;
        let target = destination.join(relative);
        let file_type = entry.file_type();

        if file_type.is_dir() {
            fs::create_dir(&target)?;
            directory_permissions.push((target, entry.metadata().map_err(io::Error::other)?.permissions()));
        } else if file_type.is_symlink() {
            copy_symlink(entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    for (directory, permissions) in directory_permissions.into_iter().rev() {
        fs::set_permissions(directory, permissions)?;
    }
    Ok(())
}

#[cfg(unix)]
fn copy_symlink(source: &Path, target: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(fs::read_link(source)?, target)
}

#[cfg(not(unix))]
fn copy_symlink(source: &Path, target: &Path) -> io::Result<()> {
    // Creating symlinks needs extra privileges on Windows; copy what it points to.
    copy_item(source, target)
}

fn move_item(source: &Path, destination: &Path, config: &Config) -> io::Result<()> {
    match fs::rename(source, destination) {
        // rename cannot cross filesystems; copy, then remove the original.
        Err(e) if e.kind() == io::ErrorKind::CrossesDevices => {
            copy_item(source, destination)?;
            // The data is safely at the destination now, so failing to remove
            // the original must not undo the move (it may be partly removed).
            if let Err(e) = remove_item(source) {
                config.print_warning(&format!(
                    "Moved to '{}', but could not fully remove the original '{}': {}",
                    destination.display(),
                    source.display(),
                    e
                ));
            }
            Ok(())
        }
        result => result,
    }
}

fn remove_item(path: &Path) -> io::Result<()> {
    if path.symlink_metadata()?.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

pub fn handle_list_command(_absolute: bool, relative: bool, config: &Config) -> Result<(), String> {
    let paths = selection_store::read_selected_paths()?;

    if paths.is_empty() {
        config.print_normal("No files currently selected.");
        return Ok(());
    }

    config.print_normal("Currently selected files:");
    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let current_dir = current_dir.canonicalize().unwrap_or(current_dir);

    for path in paths {
        let display_path = if relative {
            path.strip_prefix(&current_dir).unwrap_or(&path)
        } else {
            &path
        };
        let note = if path.symlink_metadata().is_err() { "  (missing)" } else { "" };
        config.print_normal(&format!("  {}{}", display_path.display(), note));
    }
    Ok(())
}

pub fn handle_deselect_command(items: &[PathBuf], missing: bool, config: &Config) -> Result<(), String> {
    let mut paths = selection_store::read_selected_paths()?;
    let before = paths.len();

    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    for item in items {
        // Selected paths are canonical; an item that no longer exists can only
        // be matched by its normalized absolute form.
        let target = current_dir.join(item);
        let target = target.canonicalize().unwrap_or_else(|_| normalize(&target));
        if paths.contains(&target) {
            paths.retain(|p| *p != target);
            config.print_verbose(&format!("Deselected '{}'", target.display()));
        } else {
            config.print_warning(&format!("'{}' is not selected", target.display()));
        }
    }
    if missing {
        paths.retain(|p| {
            let exists = p.symlink_metadata().is_ok();
            if !exists {
                config.print_verbose(&format!("Deselected missing '{}'", p.display()));
            }
            exists
        });
    }

    selection_store::replace_selected_paths(&paths)?;
    config.print_normal(&format!(
        "Deselected {}; {} remaining.",
        plural(before - paths.len(), "item"),
        paths.len()
    ));
    Ok(())
}

pub fn handle_clear_command(config: &Config) -> Result<(), String> {
    selection_store::clear_selection_file()?;
    config.print_normal("Selection cleared.");
    Ok(())
}

/// Resolves '.' and '..' without touching the filesystem.
fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                result.pop();
            }
            other => result.push(other),
        }
    }
    result
}
