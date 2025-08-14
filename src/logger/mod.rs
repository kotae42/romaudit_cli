// src/logger/mod.rs - Logger module

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::collections::HashSet;

use crate::error::Result;
use crate::types::{ScanResult, KnownRoms};
use crate::config::Config;

pub struct Logger {
    config: Config,
}

impl Logger {
    pub fn new(config: Config) -> Self {
        Logger { config }
    }
    
    pub fn write_logs(
        &self,
        results: &ScanResult,
        all_games: &HashSet<String>,
        known_roms: &KnownRoms,
        games_needing_folders: &HashSet<String>,
    ) -> Result<()> {
        self.write_have_log(&results.have, all_games)?;
        self.write_missing_log(&results.missing, all_games)?;
        
        if !results.shared_roms.is_empty() {
            self.write_shared_log(&results.shared_roms, known_roms)?;
        }
        
        if !games_needing_folders.is_empty() {
            self.write_folders_log(games_needing_folders)?;
        }
        
        self.print_summary(results, all_games, games_needing_folders);
        
        Ok(())
    }
    
    fn write_have_log(&self, have: &HashSet<String>, all_games: &HashSet<String>) -> Result<()> {
        let have_log = Path::new(&self.config.logs_dir).join("have.txt");
        let mut have_file = File::create(&have_log)?;
        
        writeln!(have_file, "ROMs Found: {} / {}", have.len(), all_games.len())?;
        writeln!(have_file)?;
        
        let mut have_list: Vec<_> = have.iter().collect();
        have_list.sort();
        for name in have_list {
            writeln!(have_file, "{}", name)?;
        }
        
        Ok(())
    }
    
    fn write_missing_log(&self, missing: &HashSet<String>, all_games: &HashSet<String>) -> Result<()> {
        let missing_log = Path::new(&self.config.logs_dir).join("missing.txt");
        let mut missing_file = File::create(&missing_log)?;
        
        writeln!(missing_file, "Missing ROMs: {} / {}", missing.len(), all_games.len())?;
        writeln!(missing_file)?;
        
        let mut missing_list: Vec<_> = missing.iter().collect();
        missing_list.sort();
        for name in missing_list {
            writeln!(missing_file, "{}", name)?;
        }
        
        Ok(())
    }
    
    fn write_shared_log(
        &self,
        shared_roms: &std::collections::HashMap<String, Vec<String>>,
        known_roms: &KnownRoms,
    ) -> Result<()> {
        let shared_log = Path::new(&self.config.logs_dir).join("shared.txt");
        let mut shared_file = File::create(&shared_log)?;
        
        writeln!(shared_file, "Shared ROMs (same file content used by multiple games - each has its own copy):")?;
        writeln!(shared_file, "===============================================================================")?;
        writeln!(shared_file)?;
        
        let mut shared_list: Vec<_> = shared_roms.iter().collect();
        shared_list.sort_by_key(|(hash, _)| *hash);
        
        for (hash, games) in &shared_list {
            writeln!(shared_file, "Hash: {}", hash)?;
            writeln!(shared_file, "Shared by {} games:", games.len())?;
            
            // Try to find the ROM name(s) for this hash
            let mut rom_names = HashSet::new();
            if let Some(entries) = known_roms.get(*hash) {
                for (_, rom_name) in entries {
                    rom_names.insert(rom_name.clone());
                }
            }
            
            if !rom_names.is_empty() {
                writeln!(shared_file, "ROM name(s): {}", 
                    rom_names.iter().cloned().collect::<Vec<_>>().join(", "))?;
            }
            
            writeln!(shared_file)?;
            
            // Sort games alphabetically and display each on its own line
            let mut sorted_games = (*games).clone();
            sorted_games.sort();
            
            for game in sorted_games {
                writeln!(shared_file, "  - {}", game)?;
            }
            
            writeln!(shared_file)?;
            writeln!(shared_file, "--------------------------------------------------------------------------------")?;
            writeln!(shared_file)?;
        }
        
        // Add summary at the end
        let total_shared_files = shared_list.len();
        let total_affected_games: HashSet<&String> = shared_list
            .iter()
            .flat_map(|(_, games)| games.iter())
            .collect();
        
        writeln!(shared_file, "Summary:")?;
        writeln!(shared_file, "Total shared files: {}", total_shared_files)?;
        writeln!(shared_file, "Total games affected: {}", total_affected_games.len())?;
        
        Ok(())
    }
    
    fn write_folders_log(&self, games_needing_folders: &HashSet<String>) -> Result<()> {
        let folders_log = Path::new(&self.config.logs_dir).join("folders.txt");
        let mut folders_file = File::create(&folders_log)?;
        
        writeln!(folders_file, "Games stored in subfolders:")?;
        writeln!(folders_file, "- Games with multiple ROM files")?;
        writeln!(folders_file, "- Single ROM games where ROM filename differs from game name")?;
        writeln!(folders_file, "  (ROM keeps its original name from DAT, placed in game-named folder)")?;
        writeln!(folders_file)?;
        
        let mut folders_list: Vec<_> = games_needing_folders.iter().collect();
        folders_list.sort();
        
        for game in folders_list {
            writeln!(folders_file, "{}", game)?;
        }
        
        Ok(())
    }
    
    fn print_summary(
        &self,
        results: &ScanResult,
        all_games: &HashSet<String>,
        games_needing_folders: &HashSet<String>,
    ) {
        println!("Audit complete!");
        println!("Found: {} / {} ROMs ({:.1}%)",
            results.have.len(),
            all_games.len(),
            (results.have.len() as f64 / all_games.len() as f64) * 100.0
        );
        println!("Duplicates: {}, Unknown: {}", 
            results.duplicate.len(), 
            results.unknown.len()
        );
        
        if !results.shared_roms.is_empty() {
            println!("Shared ROMs: {} (check {}/shared.txt for details)",
                results.shared_roms.len(), self.config.logs_dir);
        }
        
        if !games_needing_folders.is_empty() {
            println!("Games in folders: {} (check {}/folders.txt for details)",
                games_needing_folders.len(), self.config.logs_dir);
        }
        
        println!("Check the {}/ directory for detailed results.", self.config.logs_dir);
    }
}