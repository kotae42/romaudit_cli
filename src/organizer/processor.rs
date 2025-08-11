// src/organizer/processor.rs - File processing

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use crate::error::Result;
use crate::types::{FileHash, DatType, KnownRoms};
use crate::config::Config;
use super::folders;

pub enum ProcessResult {
    Organized(String),  // Game name
    Duplicate(String),  // Filename
    Unknown(String),    // Filename
}

/// Process a single file based on its hash matches
pub fn process_file(
    file_hash: FileHash,
    games_with_files: &HashSet<String>,
    games_needing_folders: &HashSet<String>,
    config: &Config,
    dat_type: &DatType,
    parent_clone_map: &HashMap<String, String>,
    duplicate_dir: &mut Option<PathBuf>,
    unknown_dir: &mut Option<PathBuf>,
    known_roms: &mut KnownRoms,
    is_mame_dat: bool,  // NEW parameter
) -> Result<ProcessResult> {
    let filename = file_hash.path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    // Note: file_hash.md5 and file_hash.crc are available here if needed
    // Currently we use sha1 for the known_roms database
    
    if !file_hash.matching_entries.is_empty() {
        // Filter to only process games that are present in our collection
        let mut entries_for_present_games = file_hash.matching_entries
            .iter()
            .filter(|entry| games_with_files.contains(&entry.game))
            .cloned()
            .collect::<Vec<_>>();
        
        if !entries_for_present_games.is_empty() {
            // Apply DAT type-specific filtering only for non-MAME DATs
            if !is_mame_dat && (*dat_type == DatType::Split || *dat_type == DatType::Merged) {
                entries_for_present_games = filter_entries_by_dat_type(
                    entries_for_present_games,
                    dat_type,
                    parent_clone_map,
                );
            }
            // For MAME DATs, use entries exactly as specified in XML
            
            // Process placements
            let mut first_placement = true;
            let mut source_path = file_hash.path.clone();
            let mut placed_successfully = false;
            let mut organized_game = String::new();
            
            for rom_entry in &entries_for_present_games {
                let game_name = &rom_entry.game;
                let needs_folder = games_needing_folders.contains(game_name) ||
                                   rom_entry.name.contains('\\') || 
                                   rom_entry.name.contains('/');
                
                let new_path = calculate_rom_path(
                    &rom_entry.name,
                    game_name,
                    needs_folder,
                    &config.rom_dir,
                )?;
                
                if new_path.exists() {
                    // File already exists at destination
                    continue;
                }
                
                // For MAME DATs, only place files once per game (no duplication)
                if is_mame_dat && !first_placement {
                    // For MAME, each game should have its own copy already specified
                    // We don't duplicate files between games
                    continue;
                }
                
                if first_placement {
                    // Move the file for the first placement
                    fs::rename(&file_hash.path, &new_path)?;
                    source_path = new_path.clone();
                    first_placement = false;
                    placed_successfully = true;
                    organized_game = game_name.clone();
                } else if !is_mame_dat {
                    // Copy the file for subsequent placements (only for non-MAME)
                    fs::copy(&source_path, &new_path)?;
                }
                
                // Add to known ROMs
                known_roms.entry(file_hash.sha1.clone())
                    .or_insert_with(Vec::new)
                    .push((game_name.clone(), rom_entry.name.clone()));
            }
            
            if placed_successfully {
                return Ok(ProcessResult::Organized(organized_game));
            } else {
                // All destinations existed, treat as duplicate
                if duplicate_dir.is_none() {
                    *duplicate_dir = Some(folders::create_next_folder(&config.duplicate_prefix)?);
                }
                let dup_path = duplicate_dir.as_ref().unwrap().join(&filename);
                fs::rename(&file_hash.path, &dup_path)?;
                return Ok(ProcessResult::Duplicate(filename));
            }
        } else {
            // ROM is in DAT but not for any games in our collection
            if unknown_dir.is_none() {
                *unknown_dir = Some(folders::create_next_folder(&config.unknown_prefix)?);
            }
            let unk_path = unknown_dir.as_ref().unwrap().join(&filename);
            fs::rename(&file_hash.path, &unk_path)?;
            return Ok(ProcessResult::Unknown(filename));
        }
    } else {
        // Unknown ROM - not in DAT at all
        if unknown_dir.is_none() {
            *unknown_dir = Some(folders::create_next_folder(&config.unknown_prefix)?);
        }
        let unk_path = unknown_dir.as_ref().unwrap().join(&filename);
        fs::rename(&file_hash.path, &unk_path)?;
        return Ok(ProcessResult::Unknown(filename));
    }
}

/// Filter entries based on DAT type rules (for non-MAME DATs only)
fn filter_entries_by_dat_type(
    entries: Vec<crate::types::RomEntry>,
    dat_type: &DatType,
    parent_clone_map: &HashMap<String, String>,
) -> Vec<crate::types::RomEntry> {
    let mut parent_games = HashSet::new();
    let mut clone_games = HashSet::new();
    
    for entry in &entries {
        if parent_clone_map.contains_key(&entry.game) {
            clone_games.insert(entry.game.clone());
        } else {
            parent_games.insert(entry.game.clone());
        }
    }
    
    match dat_type {
        DatType::Split => {
            // If ROM exists in parent, don't duplicate to clones
            if !parent_games.is_empty() {
                entries.into_iter()
                    .filter(|e| !parent_clone_map.contains_key(&e.game))
                    .collect()
            } else {
                entries
            }
        }
        DatType::Merged => {
            // Clone ROMs stay with parent only
            entries.into_iter()
                .filter(|e| !parent_clone_map.contains_key(&e.game))
                .collect()
        }
        _ => entries,
    }
}

/// Calculate the destination path for a ROM
fn calculate_rom_path(
    rom_name: &str,
    game_name: &str,
    needs_folder: bool,
    rom_dir: &str,
) -> Result<PathBuf> {
    let new_path = if needs_folder {
        if rom_name.contains('\\') || rom_name.contains('/') {
            // Preserve internal folder structure
            let mut path_parts = Path::new(rom_dir).join(game_name);
            for part in rom_name.split(&['\\', '/'][..]) {
                path_parts = path_parts.join(part);
            }
            if let Some(parent) = path_parts.parent() {
                fs::create_dir_all(parent)?;
            }
            path_parts
        } else {
            let game_dir = Path::new(rom_dir).join(game_name);
            fs::create_dir_all(&game_dir)?;
            game_dir.join(rom_name)
        }
    } else {
        fs::create_dir_all(rom_dir)?;
        Path::new(rom_dir).join(rom_name)
    };
    
    Ok(new_path)
}
