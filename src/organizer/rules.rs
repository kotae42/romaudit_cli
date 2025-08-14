// src/organizer/rules.rs - Helper functions for organizers.

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashSet, HashMap};

use crate::error::Result;
use crate::types::{RomEntry, RomDb};
use crate::config::Config;
use super::folders;

pub fn identify_games_needing_folders(rom_db: &RomDb, _config: &Config) -> HashSet<String> {
    let mut games_needing_folders = HashSet::new();
    let mut game_rom_info: HashMap<String, Vec<RomEntry>> = HashMap::new();
    for entries in rom_db.values() {
        for entry in entries {
            game_rom_info.entry(entry.game.clone()).or_default().push(entry.clone());
        }
    }
    for (game_name, entries) in game_rom_info {
        if entries.len() > 1 { games_needing_folders.insert(game_name); continue; }
        if let Some(entry) = entries.first() {
            if entry.is_disk { games_needing_folders.insert(game_name); continue; }
            if entry.name.contains('/') || entry.name.contains('\\') { games_needing_folders.insert(game_name); continue; }
            let rom_stem = Path::new(&entry.name).file_stem().and_then(|s| s.to_str()).unwrap_or(&entry.name);
            if rom_stem != game_name { games_needing_folders.insert(game_name); }
        }
    }
    games_needing_folders
}

pub fn move_to_folder(source_path: &Path, dest_dir: &mut Option<PathBuf>, prefix: &str) -> Result<()> {
    if dest_dir.is_none() { *dest_dir = Some(folders::create_next_folder(prefix)?); }
    if let Some(dir) = dest_dir {
        if let Some(filename) = source_path.file_name() {
            let _ = fs::rename(source_path, dir.join(filename));
        }
    }
    Ok(())
}

pub fn calculate_rom_path(entry: &RomEntry, games_needing_folders: &HashSet<String>, rom_dir: &str) -> Result<PathBuf> {
    let needs_folder = games_needing_folders.contains(&entry.game);
    let base_dir = Path::new(rom_dir);
    let game_dir = base_dir.join(&entry.game);
    let final_path = if entry.is_disk {
        game_dir.join(&entry.name).join(format!("{}.chd", &entry.name))
    } else if needs_folder {
        game_dir.join(&entry.name)
    } else {
        base_dir.join(&entry.name)
    };
    Ok(final_path)
}
