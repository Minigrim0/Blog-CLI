use std::fs::DirBuilder;
use std::path::Path;
use std::{io, fs};

use log::info;

/// Creates a directory at the given path if it does not exist.
pub fn create_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        info!("Creating path: {}", path.display());
        DirBuilder::new()
            .recursive(true)
            .create(path)
            .map_err(|e| format!("Failed to create directory: {e}"))?;
    }

    Ok(())
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}
