// src/organizer/rules.rs - Organization rules

use std::collections::HashSet;
use std::path::Path;
use crate::config::Config;
use crate::types::RomDb;

/// Identify games that need folders based on various rules
pub fn identify_games_needing_folders(
    rom_db: &RomDb,
    config: &Config,
) -> HashSet<String> {
    let mut games_needing_folders = HashSet::new();
    
    // Count ROMs per game
    let mut game_rom_counts: std::collections::HashMap<String, HashSet<String>> = std::collections::HashMap::new();
    
    for rom_entries in rom_db.values() {
        for rom_entry in rom_entries {
            game_rom_counts
                .entry(rom_entry.game.clone())
                .or_insert_with(HashSet::new)
                .insert(rom_entry.name.clone());
        }
    }
    
    // Check each game to determine if it needs a folder
    for (game_name, rom_names) in game_rom_counts {
        let rom_count = rom_names.len();
        
        // Two cases for needing folders:
        // 1. Games with multiple ROMs always get folders
        // 2. Single ROM games where ROM name doesn't match game name
        if rom_count > 1 {
            games_needing_folders.insert(game_name);
        } else if rom_count == 1 {
            // For single ROM games, check if the ROM name matches the game name
            if let Some(rom_name) = rom_names.iter().next() {
                if !is_rom_name_similar_to_game(&game_name, rom_name, config) {
                    games_needing_folders.insert(game_name);
                }
            }
        }
    }
    
    games_needing_folders
}

/// Check if a ROM name is similar enough to the game name.
/// The new logic is simple: if the ROM's stem (name without extension)
/// exactly matches the game name, they are similar. Otherwise, they are not.
pub fn is_rom_name_similar_to_game(game_name: &str, rom_name: &str, _config: &Config) -> bool {
    Path::new(rom_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .map_or(false, |stem| stem == game_name)
}