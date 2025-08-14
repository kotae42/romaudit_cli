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

/// Check if a ROM name is similar enough to the game name
pub fn is_rom_name_similar_to_game(game_name: &str, rom_name: &str, config: &Config) -> bool {
    // First, get the ROM name without extension
    let rom_without_ext = Path::new(rom_name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(rom_name);

    // If the ROM name without extension exactly matches the game name, they're definitely similar!
    if rom_without_ext == game_name {
        return true;
    }

    // Special handling for very different names
    let game_has_spaces = game_name.contains(' ');
    let rom_has_spaces = rom_without_ext.contains(' ');
    let rom_has_separators = rom_without_ext.contains('_') || rom_without_ext.contains('.');

    if game_has_spaces && !rom_has_spaces && rom_has_separators {
        // Cases like "[BIOS] Play-Yan Micro Key File (Japan)" vs "play_yanmicro.ini"
        return false;
    }

    // If the ROM name is all uppercase and the game name isn't, they're different
    if rom_without_ext.chars().any(|c| c.is_alphabetic() && c.is_uppercase()) &&
       rom_without_ext == rom_without_ext.to_uppercase() &&
       game_name != game_name.to_uppercase() {
        // Cases like "MEMORY.ASF" vs "Memory (Japan)"
        return false;
    }

    // If the game name has additional context (parentheses/brackets) that the ROM lacks
    let game_has_context = game_name.contains('(') || game_name.contains('[');
    let rom_has_context = rom_without_ext.contains('(') || rom_without_ext.contains('[');

    if game_has_context && !rom_has_context {
        // Extract base names for comparison
        let game_base = extract_base_name(game_name);
        let rom_base = extract_base_name(rom_name);

        // For short names or names with very different formatting, be strict
        if game_base.len() <= 10 || rom_base.len() <= 10 {
            // Require exact match for short names
            return game_base == rom_base;
        }

        // For longer names, check if they're meaningfully similar
        let game_lower = game_base.to_lowercase();
        let rom_lower = rom_base.to_lowercase();

        // If bases are completely different, not similar
        if !game_lower.contains(&rom_lower) && !rom_lower.contains(&game_lower) {
            return false;
        }
    }

    // Standard similarity checks for other cases
    let game_base = extract_base_name(game_name);
    let rom_base = extract_base_name(rom_name);

    // 1. Exact match
    if game_base == rom_base {
        return true;
    }

    // 2. Case-insensitive match for longer names only
    if game_base.len() > 8 && game_base.eq_ignore_ascii_case(&rom_base) {
        return true;
    }

    // 3. One contains the other (for longer names)
    if game_base.len() > 5 && rom_base.len() > 5 {
        let game_lower = game_base.to_lowercase();
        let rom_lower = rom_base.to_lowercase();

        if game_lower.contains(&rom_lower) || rom_lower.contains(&game_lower) {
            return true;
        }
    }

    // 4. Check word similarity for multi-word names
    let game_words = extract_significant_words(&game_base.to_lowercase(), &config.stop_words);
    let rom_words = extract_significant_words(&rom_base.to_lowercase(), &config.stop_words);

    if game_words.len() >= 2 && rom_words.len() >= 2 {
        let common_words: HashSet<_> = game_words.intersection(&rom_words).collect();
        let similarity_ratio = common_words.len() as f32 / game_words.len().min(rom_words.len()) as f32;

        return similarity_ratio >= 0.7; // Higher threshold for multi-word names
    }

    false
}

fn extract_base_name(name: &str) -> String {
    // Remove file extension
    let without_ext = Path::new(name)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(name);

    // Remove common suffixes in parentheses or brackets
    let mut base = without_ext.to_string();

    // Remove brackets and their contents first (like [BIOS])
    while let Some(start) = base.find('[') {
        if let Some(end) = base[start..].find(']') {
            let end_pos = start + end + 1;
            // Remove the bracket and any trailing space
            if end_pos < base.len() && base.chars().nth(end_pos) == Some(' ') {
                base.replace_range(start..end_pos + 1, "");
            } else {
                base.replace_range(start..end_pos, "");
            }
        } else {
            break;
        }
    }

    // Remove parentheses and their contents (like regions)
    if let Some(pos) = base.find('(') {
        base.truncate(pos);
    }

    base.trim().to_string()
}

fn extract_significant_words(text: &str, stop_words: &[String]) -> HashSet<String> {
    // Split on non-alphanumeric characters and filter out configured stop words
    text.split(|c: char| !c.is_alphanumeric())
        .map(|s| s.trim())
        .filter(|s| s.len() > 2 && !stop_words.contains(&s.to_string()))
        .map(|s| s.to_string())
        .collect()
}