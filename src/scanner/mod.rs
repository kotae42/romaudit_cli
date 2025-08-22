// src/scanner/mod.rs - Scanner module root

pub mod hasher;
pub mod hasher_optimized;
pub mod collector;
pub mod incremental;

use std::path::Path;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use crate::error::Result;
use crate::types::{FileHash, RomDb, RomEntry};
use crate::config::Config;
use crate::cache::HashCache;

pub struct Scanner {
    config: Config,
    interrupted: Arc<AtomicBool>,
    cache: HashCache,
    incremental_state: incremental::IncrementalScanState,
}

impl Scanner {
    pub fn new(config: Config, interrupted: Arc<AtomicBool>) -> Self {
        let cache = HashCache::load().unwrap_or_else(|_| HashCache::new());
        let incremental_state = incremental::IncrementalScanState::load()
            .unwrap_or_else(|_| incremental::IncrementalScanState::new());
        
        Scanner { 
            config, 
            interrupted,
            cache,
            incremental_state,
        }
    }
    
    /// Scan files and calculate hashes, identifying which games are present
    pub fn scan_files(
        &mut self,
        scan_path: &Path,
        rom_db: &RomDb,
    ) -> Result<(Vec<FileHash>, HashSet<String>)> {
        // Collect files
        let all_files = collector::collect_files_recursively(scan_path, &self.config)?;
        
        // Determine which files need scanning (incremental)
        let files_to_scan = self.incremental_state.get_files_to_scan(&all_files);
        let using_incremental = files_to_scan.len() < all_files.len();
        
        if using_incremental {
            println!("Incremental scan: {} total files, {} need scanning, {} cached",
                     all_files.len(), files_to_scan.len(), all_files.len() - files_to_scan.len());
        } else {
            println!("Scanning {} files to identify games and calculate hashes...", all_files.len());
        }
        println!("This may take a while for large collections.");
        
        let bar = ProgressBar::new(files_to_scan.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} [{eta_precise}]"
            ).unwrap(),
        );
        
        let mut file_hashes = Vec::new();
        let mut games_with_files = HashSet::new();
        
        // First, add cached results for files that haven't changed
        for file in &all_files {
            if !files_to_scan.contains(file) {
                // Use cached data
                if let Some(cached_info) = self.cache.get(file) {
                    let matching_entries = find_matching_entries(rom_db, &cached_info.sha1, &cached_info.md5, &cached_info.crc);
                    
                    for entry in &matching_entries {
                        games_with_files.insert(entry.game.clone());
                    }
                    
                    file_hashes.push(FileHash {
                        path: file.clone(),
                        sha1: cached_info.sha1,
                        md5: cached_info.md5,
                        crc: cached_info.crc,
                        matching_entries,
                    });
                }
            }
        }
        
        // Now scan only the files that need it
        for file in files_to_scan {
            // Check for interruption
            if self.interrupted.load(Ordering::Relaxed) {
                bar.finish_with_message("Interrupted by user!");
                println!("\nProcess interrupted during scanning.");
                return Ok((file_hashes, games_with_files));
            }
            
            let filename = file.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            bar.set_message(format!("Hashing: {}", 
                if filename.len() > 40 { 
                    format!("...{}", &filename[filename.len()-37..]) 
                } else { 
                    filename.clone() 
                }
            ));
            
            // Calculate hashes with optimizations
            let (sha1, md5, crc) = hasher_optimized::calculate_hashes_cached(
                &file, 
                self.config.buffer_size,
                &mut self.cache
            )?;
            
            // Update incremental state
            self.incremental_state.update_file(&file, sha1.clone())?;
            
            // Find matching ROM entries
            let matching_entries = find_matching_entries(rom_db, &sha1, &md5, &crc);
            
            // Track which games have files present
            for entry in &matching_entries {
                games_with_files.insert(entry.game.clone());
            }
            
            file_hashes.push(FileHash {
                path: file,
                sha1,
                md5,
                crc,
                matching_entries,
            });
            
            bar.inc(1);
        }
        
        bar.finish_with_message(format!("Found {} games with files present", games_with_files.len()));
        
        // Save cache and incremental state
        self.cache.save()?;
        self.incremental_state.save()?;
        
        Ok((file_hashes, games_with_files))
    }
}

/// Find all ROM entries matching the given hashes
fn find_matching_entries(rom_db: &RomDb, sha1: &str, md5: &str, crc: &str) -> Vec<RomEntry> {
    [sha1, md5, crc]
        .iter()
        .filter_map(|hash| rom_db.get(*hash))
        .flatten()
        .cloned()
        .collect()
}