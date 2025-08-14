// src/organizer/mod.rs - Organizer module root

pub mod rules;
pub mod folders;
pub mod processor;

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use indicatif::{ProgressBar, ProgressStyle};

use crate::error::Result;
use crate::types::{FileHash, ScanResult, KnownRoms, RomDb};
use crate::config::Config;

pub struct Organizer {
    config: Config,
    games_needing_folders: HashSet<String>,
    interrupted: Arc<AtomicBool>,
}

impl Organizer {
    pub fn new(
        config: Config,
        rom_db: &RomDb,
        interrupted: Arc<AtomicBool>,
    ) -> Self {
        let games_needing_folders = rules::identify_games_needing_folders(rom_db, &config);
        
        Organizer {
            config,
            games_needing_folders,
            interrupted,
        }
    }
    
    /// Get the set of games needing folders
    pub fn games_needing_folders(&self) -> &HashSet<String> {
        &self.games_needing_folders
    }
    
    /// Organize files based on DAT information
    pub fn organize_files(
        &self,
        file_hashes: Vec<FileHash>,
        games_with_files: &HashSet<String>,
        known_roms: &mut KnownRoms,
    ) -> Result<ScanResult> {
        let mut result = ScanResult {
            have: HashSet::new(),
            missing: HashSet::new(),
            duplicate: Vec::new(),
            unknown: Vec::new(),
            shared_roms: HashMap::new(),
        };
        
        // Build initial have set from known_roms
        for entries in known_roms.values() {
            for (game, _) in entries {
                result.have.insert(game.clone());
            }
        }
        
        // Create necessary directories
        std::fs::create_dir_all(&self.config.rom_dir)?;
        std::fs::create_dir_all(&self.config.logs_dir)?;
        
        println!("Organizing ROMs for {} games...", games_with_files.len());
        
        // Set up progress bar
        let bar = ProgressBar::new(file_hashes.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
            ).unwrap(),
        );
        bar.set_message("Organizing files...");
        
        let mut duplicate_dir = None;
        let mut unknown_dir = None;
        
        // Process files
        for file_hash in file_hashes {
            // Check for interruption
            if self.interrupted.load(Ordering::Relaxed) {
                bar.finish_with_message("Interrupted by user!");
                println!("\nProcess interrupted. Partial results may have been saved.");
                return Ok(result);
            }
            
            let filename = file_hash.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            bar.set_message(format!("Processing: {}", 
                if filename.len() > 40 { 
                    format!("...{}", &filename[filename.len()-37..]) 
                } else { 
                    filename.clone() 
                }
            ));
            
            // Process the file
            let processed = processor::process_file(
                file_hash,
                games_with_files,
                &self.games_needing_folders,
                &self.config,
                &mut duplicate_dir,
                &mut unknown_dir,
                known_roms,
            )?;
            
            // Update result
            match processed {
                processor::ProcessResult::Organized(game) => {
                    result.have.insert(game);
                }
                processor::ProcessResult::Duplicate(file) => {
                    result.duplicate.push(file);
                }
                processor::ProcessResult::Unknown(file) => {
                    result.unknown.push(file);
                }
            }
            
            bar.inc(1);
        }
        
        bar.finish_with_message("Organization complete!");
        
        // Track shared ROMs
        for (hash, entries) in known_roms.iter() {
            if entries.len() > 1 {
                let games: Vec<String> = entries.iter()
                    .map(|(game, _)| game.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                
                if games.len() > 1 {
                    result.shared_roms.insert(hash.clone(), games);
                }
            }
        }
        
        Ok(result)
    }
}