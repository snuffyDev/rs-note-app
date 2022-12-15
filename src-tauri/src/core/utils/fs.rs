use std::{io::Write, path::PathBuf};

use atomicwrites::{AtomicFile, OverwriteBehavior};

pub fn ensure_parent_exists(file_path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            throw!("Error creating parent folder: {}", e.to_string());
        }
    }
    Ok(())
}

pub fn write_atomically(file_path: &PathBuf, buf: serde_json::Value) -> Result<(), String> {
    ensure_parent_exists(&file_path)?;
    let af = AtomicFile::new(&file_path, OverwriteBehavior::AllowOverwrite);
    match af.write(|f| f.write_all(&buf.to_string().into_bytes())) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
