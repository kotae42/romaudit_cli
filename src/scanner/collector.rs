// src/scanner/collector.rs - File collection

use std::fs;
use std::path::{Path, PathBuf};
use crate::config::Config;
use crate::error::{Result, RomAuditError};

/// Recursively collect all files to be processed
pub fn collect_files_recursively(dir: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_recursive_impl(dir, config, &mut files)?;
    files.sort_by_key(|p| p.to_string_lossy().to_lowercase());
    Ok(files)
}

fn collect_files_recursive_impl(dir: &Path, config: &Config, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            if should_process_file(&path, config)? {
                files.push(path);
            }
        } else if path.is_dir() {
            if !is_generated_directory(&path, config) {
                collect_files_recursive_impl(&path, config, files)?;
            }
        }
    }
    Ok(())
}

/// Check if a file should be processed
fn should_process_file(path: &Path, config: &Config) -> Result<bool> {
    let file_name = path.file_name()
        .ok_or_else(|| RomAuditError::InvalidPath(path.to_string_lossy().to_string()))?
        .to_string_lossy();

    // Skip DAT/XML files
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if ext.eq_ignore_ascii_case("dat") || ext.eq_ignore_ascii_case("xml") {
            return Ok(false);
        }
    }

    // Skip database and temp files
    if file_name == config.db_file || file_name.ends_with(".tmp") {
        return Ok(false);
    }

    // Skip if in generated directory
    if is_generated_directory(path, config) {
        return Ok(false);
    }

    Ok(true)
}

/// Check if a path is within a generated directory
pub fn is_generated_directory(path: &Path, config: &Config) -> bool {
    if let Some(name) = path.file_name() {
        let name_str = name.to_string_lossy();

        // Check if this is one of our generated directories
        if name_str == config.rom_dir ||
           name_str == config.logs_dir ||
           name_str.starts_with(&config.duplicate_prefix) ||
           name_str.starts_with(&config.unknown_prefix) {
            return true;
        }

        // Also check ancestors
        path.ancestors().any(|ancestor| {
            if let Some(ancestor_name) = ancestor.file_name() {
                let ancestor_str = ancestor_name.to_string_lossy();
                ancestor_str == config.rom_dir ||
                ancestor_str == config.logs_dir ||
                ancestor_str.starts_with(&config.duplicate_prefix) ||
                ancestor_str.starts_with(&config.unknown_prefix)
            } else {
                false
            }
        })
    } else {
        false
    }
}