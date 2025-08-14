// src/organizer/mame.rs - MAME-specific organizer logic.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::fs;

use indicatif::{ProgressBar, ProgressStyle};

use crate::config::Config;
use crate::error::Result;
use crate::types::{DatType, FileHash, KnownRoms, RomDb, RomEntry, ScanResult};
use super::plugin::OrganizerPlugin;
use super::rules;

pub struct MameOrganizer {
    config: Config,
    dat_type: DatType,
    parent_clone_map: HashMap<String, String>,
    games_needing_folders: HashSet<String>,
    interrupted: Arc<AtomicBool>,
}

impl MameOrganizer {
    pub fn new(
        config: Config,
        dat_type: DatType,
        parent_clone_map: HashMap<String, String>,
        rom_db: &RomDb,
        interrupted: Arc<AtomicBool>,
    ) -> Self {
        let games_needing_folders = rules::identify_games_needing_folders(rom_db, &config);
        Self { config, dat_type, parent_clone_map, games_needing_folders, interrupted }
    }

    fn build_plan(&self, rom_db: &RomDb, file_hashes: &[FileHash]) -> (HashMap<String, PathBuf>, HashSet<String>) {
        let mut required_roms = HashMap::new();
        let mut user_games = HashSet::new();

        // Heuristic to determine user's collection
        let mut total_roms_per_game = HashMap::new();
        for entry in rom_db.values().flatten() {
            *total_roms_per_game.entry(entry.game.clone()).or_insert(0) += 1;
        }
        let mut user_files_per_game = HashMap::new();
        for file in file_hashes {
            for entry in &file.matching_entries {
                *user_files_per_game.entry(entry.game.clone()).or_insert(0) += 1;
            }
        }
        for (game, count) in user_files_per_game {
            let total = total_roms_per_game.get(&game).cloned().unwrap_or(1);
            if (count as f32 / total as f32) > 0.4 {
                user_games.insert(game);
            }
        }

        for game_name in &user_games {
            let roms_for_game = self.get_roms_for_game(game_name, rom_db);
            for rom in roms_for_game {
                if let Some(sha1) = &rom.hashes.sha1 {
                    let dest = rules::calculate_rom_path(&rom, &self.games_needing_folders, &self.config.rom_dir).unwrap();
                    if self.dat_type != DatType::NonMerged {
                        if let Some(_existing) = required_roms.get(sha1) {
                            if self.parent_clone_map.contains_key(&rom.game) { continue; }
                        }
                        required_roms.insert(sha1.clone(), dest);
                    } else {
                        required_roms.insert(sha1.clone(), dest);
                    }
                }
            }
        }
        (required_roms, user_games)
    }

    fn get_roms_for_game(&self, game_name: &str, rom_db: &RomDb) -> Vec<RomEntry> {
        let mut roms = Vec::new();
        for entry in rom_db.values().flatten() {
            if entry.game == game_name {
                roms.push(entry.clone());
            }
        }
        if self.dat_type == DatType::Split {
            if let Some(parent) = self.parent_clone_map.get(game_name) {
                roms.extend(self.get_roms_for_game(parent, rom_db));
            }
        }
        roms
    }
}

impl OrganizerPlugin for MameOrganizer {
    fn organize(&self, file_hashes: Vec<FileHash>, rom_db: &RomDb, _known_roms: &mut KnownRoms) -> Result<ScanResult> {
        let mut result = ScanResult::default();
        fs::create_dir_all(&self.config.rom_dir)?;
        fs::create_dir_all(&self.config.logs_dir)?;

        let (plan, user_games) = self.build_plan(rom_db, &file_hashes);
        result.have = user_games;
        println!("Planning complete. Organizing {} games.", result.have.len());

        let bar = ProgressBar::new(file_hashes.len() as u64);
        bar.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}").unwrap());
        
        let mut fulfilled = HashSet::new();
        let mut duplicate_dir = None;
        let mut unknown_dir = None;

        for file in file_hashes {
            if self.interrupted.load(Ordering::Relaxed) { break; }
            let filename = file.path.file_name().unwrap().to_str().unwrap().to_string();
            bar.set_message(format!("Processing: {}", filename));

            if let Some(dest) = plan.get(&file.sha1) {
                if fulfilled.contains(&file.sha1) {
                    rules::move_to_folder(&file.path, &mut duplicate_dir, &self.config.duplicate_prefix)?;
                    result.duplicate.push(filename);
                } else {
                    if !dest.exists() {
                        if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
                        fs::rename(&file.path, dest)?;
                    }
                    fulfilled.insert(file.sha1.clone());
                }
            } else {
                rules::move_to_folder(&file.path, &mut unknown_dir, &self.config.unknown_prefix)?;
                result.unknown.push(filename);
            }
            bar.inc(1);
        }

        bar.finish_with_message("Organization complete!");
        Ok(result)
    }
}