use std::fs::DirBuilder;
use std::path::PathBuf;

use log::info;

/// Creates a directory at the given path if it does not exist.
pub fn create_path(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        info!("Creating path: {}", path.display());
        DirBuilder::new()
            .recursive(true)
            .create(path)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    Ok(())
}
