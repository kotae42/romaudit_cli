// src/scanner/mod.rs - Scanner module root

pub mod hasher;
pub mod collector;

use std::path::Path;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use crate::error::Result;
use crate::types::{FileHash, RomDb, RomEntry};
use crate::config::Config;

pub struct Scanner {
    config: Config,
    interrupted: Arc<AtomicBool>,
}

impl Scanner {
    pub fn new(config: Config, interrupted: Arc<AtomicBool>) -> Self {
        Scanner { config, interrupted }
    }
    
    /// Scan files and calculate hashes, identifying which games are present
    pub fn scan_files(
        &self,
        scan_path: &Path,
        rom_db: &RomDb,
    ) -> Result<(Vec<FileHash>, HashSet<String>)> {
        // Collect files
        let files = collector::collect_files_recursively(scan_path, &self.config)?;
        
        // Set up progress bar
        println!("Scanning {} files to identify games and calculate hashes...", files.len());
        println!("This may take a while for large collections.");
        
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} [{eta_precise}]"
            ).unwrap(),
        );
        
        let mut file_hashes = Vec::new();
        let mut games_with_files = HashSet::new();
        
        for file in files {
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
            
            // Calculate hashes
            let (sha1, md5, crc) = hasher::calculate_hashes(&file, self.config.buffer_size)?;
            
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