// romaudit_cli - ROM Collection Management Tool
// Version 2.2.0
// Copyright (c) 2024 [Kotae42]
//
// This software is licensed for PERSONAL USE ONLY.
// Commercial use is strictly prohibited.
// See LICENSE file for full terms.

mod config;
mod error;
mod types;
mod parser;
mod scanner;
mod organizer;
mod database;
mod logger;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::path::Path;

use crate::error::Result;
use crate::config::Config;

struct RomAuditor {
    config: Config,
    parsed_dat: types::ParsedDat,
    known_roms: types::KnownRoms,
    interrupted: Arc<AtomicBool>,
}

impl RomAuditor {
    fn new(config: Config, interrupted: Arc<AtomicBool>) -> Result<Self> {
        // Find and parse DAT file
        let dat_path = parser::find_dat_file()?;
        println!("Found DAT file: {}", dat_path.display());
        
        let parsed_dat = parser::parse_dat_file(&dat_path)?;
        println!("Parsed {} games from DAT file", parsed_dat.all_games.len());
        
        // Load known ROMs database
        let known_roms = database::load_known_roms(&config.db_file)?;
        
        Ok(RomAuditor {
            config,
            parsed_dat,
            known_roms,
            interrupted,
        })
    }
    
    fn run(&mut self) -> Result<()> {
        // Scan files and calculate hashes
        let scanner = scanner::Scanner::new(self.config.clone(), self.interrupted.clone());
        let (file_hashes, games_with_files) = scanner.scan_files(
            Path::new("."),
            &self.parsed_dat.rom_db,
        )?;
        
        // Check if interrupted during scanning
        if self.interrupted.load(Ordering::Relaxed) {
            database::save_known_roms(&self.known_roms, &self.config.db_file)?;
            return Ok(());
        }
        
        // Organize files
        let organizer = organizer::Organizer::new(
            self.config.clone(),
            &self.parsed_dat.rom_db,
            self.interrupted.clone(),
        );
        
        let mut result = organizer.organize_files(
            file_hashes,
            &games_with_files,
            &mut self.known_roms,
        )?;
        
        // Update missing set
        result.missing = self.parsed_dat.all_games.clone();
        for game in &result.have {
            result.missing.remove(game);
        }
        
        // Save database
        database::save_known_roms(&self.known_roms, &self.config.db_file)?;
        
        // Write logs
        let logger = logger::Logger::new(self.config.clone());
        logger.write_logs(
            &result,
            &self.parsed_dat.all_games,
            &self.known_roms,
            organizer.games_needing_folders(),
        )?;
        
        // Clean up empty folders
        organizer::folders::remove_empty_folders(Path::new("."), &self.config)?;
        
        Ok(())
    }
}

fn main() {
    // Set up signal handling for graceful shutdown
    let interrupted = Arc::new(AtomicBool::new(false));
    let interrupted_clone = interrupted.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived interrupt signal. Cleaning up...");
        interrupted_clone.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");
    
    // Load configuration
    let config = Config::load();
    
    // Run the auditor
    match RomAuditor::new(config, interrupted).and_then(|mut auditor| auditor.run()) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}