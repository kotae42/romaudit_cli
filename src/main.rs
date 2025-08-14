// romaudit_cli - ROM Collection Management Tool

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
use crate::organizer::OrganizerPlugin;

struct RomAuditor {
    config: Config,
    parsed_dat: types::ParsedDat,
    known_roms: types::KnownRoms,
    interrupted: Arc<AtomicBool>,
}

impl RomAuditor {
    fn new(config: Config, interrupted: Arc<AtomicBool>) -> Result<Self> {
        let dat_path = parser::find_dat_file()?;
        println!("Found DAT/XML file: {}", dat_path.display());
        let parsed_dat = parser::parse_dat_file(&dat_path)?;
        
        if parsed_dat.is_mame_dat {
            println!("MAME XML detected.");
        } else {
            println!("Standard DAT detected.");
        }
        
        let known_roms = database::load_known_roms(&config.db_file)?;
        Ok(RomAuditor { config, parsed_dat, known_roms, interrupted })
    }
    
    fn run(&mut self) -> Result<()> {
        let scanner = scanner::Scanner::new(self.config.clone(), self.interrupted.clone());
        let (file_hashes, _) = scanner.scan_files(Path::new("."), &self.parsed_dat.rom_db)?;
        
        if self.interrupted.load(Ordering::Relaxed) {
            database::save_known_roms(&self.known_roms, &self.config.db_file)?;
            return Ok(());
        }
        
        let organizer: Box<dyn OrganizerPlugin> = if self.parsed_dat.is_mame_dat {
            Box::new(organizer::MameOrganizer::new(
                self.config.clone(),
                self.parsed_dat.dat_type.clone(),
                self.parsed_dat.parent_clone_map.clone(),
                &self.parsed_dat.rom_db,
                self.interrupted.clone(),
            ))
        } else {
            Box::new(organizer::StandardOrganizer::new(
                self.config.clone(),
                self.interrupted.clone(),
            ))
        };

        let mut result = organizer.organize(file_hashes, &self.parsed_dat.rom_db, &mut self.known_roms)?;
        
        result.missing = self.parsed_dat.all_games.clone();
        for game in &result.have {
            result.missing.remove(game);
        }
        
        database::save_known_roms(&self.known_roms, &self.config.db_file)?;
        
        let logger = logger::Logger::new(self.config.clone(), self.parsed_dat.dat_type.clone());
        let games_needing_folders = organizer::rules::identify_games_needing_folders(&self.parsed_dat.rom_db, &self.config);
        logger.write_logs(&result, &self.parsed_dat.all_games, &self.known_roms, &games_needing_folders)?;
        
        organizer::folders::remove_empty_folders(Path::new("."), &self.config)?;
        
        Ok(())
    }
}

fn main() {
    let interrupted = Arc::new(AtomicBool::new(false));
    let i = interrupted.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived interrupt signal. Cleaning up...");
        i.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");
    
    let config = Config::load();
    
    if let Err(e) = RomAuditor::new(config, interrupted).and_then(|mut auditor| auditor.run()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}