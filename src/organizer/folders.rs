// src/organizer/folders.rs - Folder management

use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{Result, RomAuditError};

/// Create the next numbered folder with the given prefix
pub fn create_next_folder(prefix: &str) -> Result<PathBuf> {
    for i in 1..1000 {
        let candidate = PathBuf::from(format!("{}{}", prefix, i));
        match fs::create_dir(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(RomAuditError::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        "Could not create numbered folder after 1000 attempts"
    )))
}

/// Remove empty folders recursively
pub fn remove_empty_folders(dir: &Path, config: &crate::config::Config) -> Result<()> {
    let mut folders_to_check = Vec::new();
    collect_folders_recursively(dir, &mut folders_to_check, config)?;
    
    // Sort by depth (deepest first)
    folders_to_check.sort_by(|a, b| {
        b.components().count().cmp(&a.components().count())
    });
    
    for folder in folders_to_check {
        if crate::scanner::collector::is_generated_directory(&folder, config) {
            continue;
        }
        
        if is_folder_empty(&folder)? {
            let _ = fs::remove_dir(&folder);
        }
    }
    
    Ok(())
}

fn collect_folders_recursively(
    dir: &Path,
    folders: &mut Vec<PathBuf>,
    config: &crate::config::Config,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() && !crate::scanner::collector::is_generated_directory(&path, config) {
            folders.push(path.clone());
            collect_folders_recursively(&path, folders, config)?;
        }
    }
    Ok(())
}

fn is_folder_empty(dir: &Path) -> Result<bool> {
    let mut entries = fs::read_dir(dir)?;
    Ok(entries.next().is_none())
}