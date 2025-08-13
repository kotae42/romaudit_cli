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

    // Skip DAT/XML files ONLY in the root directory (not in ROM folders)
    // Some MAME ROMs have .dat extension!
    if let Some(parent) = path.parent() {
        if parent == Path::new(".") {
            // Only skip DAT/XML in root directory
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("dat") || ext.eq_ignore_ascii_case("xml") {
                    return Ok(false);
                }
            }
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
    let Ok(current_dir) = std::env::current_dir() else { return false };
    
    let generated_dirs = [
        current_dir.join(&config.rom_dir),
        current_dir.join(&config.logs_dir),
        // Note: duplicate and unknown dirs are handled at a higher level now
        // and created inside the execution path, so we don't need to check them here.
    ];

    // Get the absolute path of the file/directory being checked
    let Ok(abs_path) = path.canonicalize() else { return false };

    // Check if the path is inside any of the generated directories
    generated_dirs.iter().any(|gen_dir| {
        let Ok(abs_gen_dir) = gen_dir.canonicalize() else { return false };
        abs_path.starts_with(abs_gen_dir)
    })
}
