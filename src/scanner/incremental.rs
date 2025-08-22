// src/scanner/incremental.rs - Incremental scanning logic

use std::collections::HashMap;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::types::{FileHash, RomDb};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileScanState {
    pub path: PathBuf,
    pub size: u64,
    pub modified: SystemTime,
    pub sha1: String,
    pub last_scanned: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IncrementalScanState {
    pub files: HashMap<PathBuf, FileScanState>,
    pub last_full_scan: Option<SystemTime>,
    pub version: u32,
}

impl IncrementalScanState {
    const STATE_VERSION: u32 = 1;
    const STATE_FILE: &'static str = ".romaudit_scan_state.json";
    
    pub fn new() -> Self {
        IncrementalScanState {
            files: HashMap::new(),
            last_full_scan: None,
            version: Self::STATE_VERSION,
        }
    }
    
    /// Load scan state from disk
    pub fn load() -> Result<Self> {
        let state_path = Path::new(Self::STATE_FILE);
        if !state_path.exists() {
            return Ok(Self::new());
        }
        
        let content = std::fs::read_to_string(state_path)?;
        let state: IncrementalScanState = serde_json::from_str(&content)?;
        
        if state.version != Self::STATE_VERSION {
            // Version mismatch, start fresh
            Ok(Self::new())
        } else {
            Ok(state)
        }
    }
    
    /// Save scan state to disk
    pub fn save(&self) -> Result<()> {
        let state_path = Path::new(Self::STATE_FILE);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(state_path, content)?;
        Ok(())
    }
    
    /// Check if a file needs to be rescanned
    pub fn needs_rescan(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }
        
        let Ok(meta) = metadata(path) else {
            return true; // If we can't read metadata, rescan to be safe
        };
        
        match self.files.get(path) {
            Some(state) => {
                // Check if file has been modified
                state.size != meta.len() || 
                state.modified != meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)
            }
            None => true, // New file, needs scanning
        }
    }
    
    /// Update the state for a scanned file
    pub fn update_file(&mut self, path: &Path, sha1: String) -> Result<()> {
        let meta = metadata(path)?;
        
        let state = FileScanState {
            path: path.to_path_buf(),
            size: meta.len(),
            modified: meta.modified()?,
            sha1,
            last_scanned: SystemTime::now(),
        };
        
        self.files.insert(path.to_path_buf(), state);
        Ok(())
    }
    
    /// Remove entries for files that no longer exist
    #[allow(dead_code)]
    pub fn cleanup(&mut self) {
        self.files.retain(|path, _| path.exists());
    }
    
    /// Get files that need scanning based on current state
    pub fn get_files_to_scan(&self, all_files: &[PathBuf]) -> Vec<PathBuf> {
        all_files.iter()
            .filter(|path| self.needs_rescan(path))
            .cloned()
            .collect()
    }
    
    /// Get statistics about the scan state
    #[allow(dead_code)]
    pub fn stats(&self) -> ScanStats {
        let total = self.files.len();
        let valid = self.files.values()
            .filter(|state| {
                state.path.exists() && {
                    if let Ok(meta) = metadata(&state.path) {
                        meta.len() == state.size
                    } else {
                        false
                    }
                }
            })
            .count();
        
        ScanStats {
            total_tracked: total,
            valid_entries: valid,
            stale_entries: total - valid,
            last_full_scan: self.last_full_scan,
        }
    }
    
    /// Mark that a full scan has been completed
    #[allow(dead_code)]
    pub fn mark_full_scan_complete(&mut self) {
        self.last_full_scan = Some(SystemTime::now());
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ScanStats {
    pub total_tracked: usize,
    pub valid_entries: usize,
    pub stale_entries: usize,
    pub last_full_scan: Option<SystemTime>,
}

/// Perform an incremental scan of files
#[allow(dead_code)]
pub async fn incremental_scan(
    base_path: &Path,
    rom_db: &RomDb,
    scan_state: &mut IncrementalScanState,
    cache: &mut crate::cache::HashCache,
    buffer_size: usize,
) -> Result<Vec<FileHash>> {
    use crate::scanner::collector::collect_files_recursively;
    use crate::scanner::hasher_optimized::calculate_hashes_cached;
    
    // Collect all files
    let config = crate::config::Config::default();
    let all_files = collect_files_recursively(base_path, &config)?;
    
    // Determine which files need scanning
    let files_to_scan = scan_state.get_files_to_scan(&all_files);
    
    println!("Incremental scan: {} files to check, {} need scanning", 
             all_files.len(), files_to_scan.len());
    
    let mut results = Vec::new();
    
    // Use cached hashes for files that haven't changed
    for file_path in &all_files {
        if !files_to_scan.contains(file_path) {
            // Use cached data
            if let Some(state) = scan_state.files.get(file_path) {
                // Look up matching ROM entries
                if let Some(entries) = rom_db.get(&state.sha1) {
                    results.push(FileHash {
                        path: file_path.clone(),
                        sha1: state.sha1.clone(),
                        md5: String::new(), // Not stored in incremental state
                        crc: String::new(), // Not stored in incremental state
                        matching_entries: entries.clone(),
                    });
                }
            }
        }
    }
    
    // Scan only the files that need it
    for file_path in files_to_scan {
        match calculate_hashes_cached(&file_path, buffer_size, cache) {
            Ok((sha1, md5, crc)) => {
                // Update scan state
                scan_state.update_file(&file_path, sha1.clone())?;
                
                // Look up matching ROM entries
                if let Some(entries) = rom_db.get(&sha1) {
                    results.push(FileHash {
                        path: file_path,
                        sha1,
                        md5,
                        crc,
                        matching_entries: entries.clone(),
                    });
                }
            }
            Err(e) => {
                eprintln!("Error scanning {}: {}", file_path.display(), e);
            }
        }
    }
    
    // Clean up stale entries
    scan_state.cleanup();
    
    // Save updated state
    scan_state.save()?;
    cache.save()?;
    
    Ok(results)
}

/// Check if incremental scanning is beneficial
#[allow(dead_code)]
pub fn should_use_incremental(total_files: usize, changed_files: usize) -> bool {
    // Use incremental if less than 20% of files have changed
    // or if we have more than 1000 files total
    changed_files < total_files / 5 || total_files > 1000
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use std::io::Write;
    
    #[test]
    fn test_needs_rescan() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rom");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test data").unwrap();
        
        let mut state = IncrementalScanState::new();
        
        // New file should need rescan
        assert!(state.needs_rescan(&file_path));
        
        // After updating, should not need rescan
        state.update_file(&file_path, "fake_sha1".to_string()).unwrap();
        assert!(!state.needs_rescan(&file_path));
        
        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(10));
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&file_path)
            .unwrap();
        file.write_all(b"more data").unwrap();
        
        // Should need rescan after modification
        assert!(state.needs_rescan(&file_path));
    }
}