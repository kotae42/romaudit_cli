// src/scanner/mod.rs - Scanner module root

pub mod hasher;
pub mod collector;

use std::path::Path;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

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
    
    /// Scan files and calculate hashes in parallel, identifying which games are present
    pub fn scan_files(
        &self,
        scan_path: &Path,
        rom_db: &RomDb,
    ) -> Result<(Vec<FileHash>, HashSet<String>)> {
        // Collect files
        let files = collector::collect_files_recursively(scan_path, &self.config)?;
        
        // Set up progress bar
        println!("Scanning {} files using multiple threads...", files.len());
        println!("This may take a while for large collections.");
        
        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg} [{eta_precise}]"
            ).unwrap(),
        );
        
        // Use a parallel iterator to process files concurrently
        let results: Vec<Result<_>> = files
            .par_iter()
            .map(|file| {
                if self.interrupted.load(Ordering::Relaxed) {
                    return Err(crate::error::RomAuditError::Io(
                        std::io::Error::new(std::io::ErrorKind::Interrupted, "User interrupted scan")
                    ));
                }

                let filename = file.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
                bar.set_message(format!("Hashing: {}", if filename.len() > 40 { format!("...{}", &filename[filename.len()-37..]) } else { filename }));

                // Calculate hashes and find matching entries
                let (sha1, md5, crc) = hasher::calculate_hashes(file, self.config.buffer_size)?;
                let matching_entries = find_matching_entries(rom_db, &sha1, &md5, &crc);
                
                // Identify games present for this file
                let games_for_file: HashSet<String> = matching_entries.iter().map(|e| e.game.clone()).collect();
                
                let file_hash = FileHash {
                    path: file.clone(),
                    sha1,
                    md5,
                    crc,
                    matching_entries,
                };
                
                bar.inc(1);
                Ok((file_hash, games_for_file))
            })
            .collect();

        // Check for interruption flag after parallel processing
        if self.interrupted.load(Ordering::Relaxed) {
            bar.finish_with_message("Interrupted by user!");
            println!("\nProcess interrupted during scanning.");
            // Return empty results as the process was aborted
            return Ok((Vec::new(), HashSet::new()));
        }

        // Merge results from all threads
        let mut file_hashes = Vec::new();
        let mut games_with_files = HashSet::new();
        for result in results {
            match result {
                Ok((file_hash, games)) => {
                    file_hashes.push(file_hash);
                    games_with_files.extend(games);
                }
                Err(e) => {
                    // If the error was due to interruption, we can ignore it here
                    if let crate::error::RomAuditError::Io(io_err) = &e {
                        if io_err.kind() == std::io::ErrorKind::Interrupted {
                            continue;
                        }
                    }
                    // For other errors, propagate them
                    return Err(e);
                }
            }
        }
        
        bar.finish_with_message(format!("Found {} games with files present", games_with_files.len()));
        
        Ok((file_hashes, games_with_files))
    }
}

/// Find all ROM entries matching the given hashes
fn find_matching_entries(rom_db: &RomDb, sha1: &str, md5: &str, crc: &str) -> Vec<RomEntry> {
    let mut unique_entries = std::collections::HashMap::new();
    for hash in [sha1, md5, crc] {
        if let Some(entries) = rom_db.get(hash) {
            for entry in entries {
                // Use the ROM name as a key to avoid duplicates from different hash matches
                unique_entries.entry(entry.name.clone()).or_insert_with(|| entry.clone());
            }
        }
    }
    unique_entries.values().cloned().collect()
}