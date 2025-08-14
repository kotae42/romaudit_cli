// src/organizer/processor.rs - File processing

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;

use crate::error::Result;
use crate::types::{FileHash, KnownRoms};
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
    duplicate_dir: &mut Option<PathBuf>,
    unknown_dir: &mut Option<PathBuf>,
    known_roms: &mut KnownRoms,
) -> Result<ProcessResult> {
    let filename = file_hash.path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    if !file_hash.matching_entries.is_empty() {
        // Filter to only process games that are present in our collection
        let entries_for_present_games = file_hash.matching_entries
            .iter()
            .filter(|entry| games_with_files.contains(&entry.game))
            .cloned()
            .collect::<Vec<_>>();
        
        if !entries_for_present_games.is_empty() {
            // Process placements
            let mut placements = 0;
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
                    rom_entry.is_disk,
                )?;
                
                if new_path.exists() {
                    // File already exists at destination
                    continue;
                }
                
                // Copy the file to all games that need it
                if fs::copy(&file_hash.path, &new_path).is_ok() {
                    placements += 1;
                    if organized_game.is_empty() {
                        organized_game = game_name.clone();
                    }

                    // Add to known ROMs
                    known_roms.entry(file_hash.sha1.clone())
                        .or_insert_with(Vec::new)
                        .push((game_name.clone(), rom_entry.name.clone()));
                }
            }

            // After all potential placements, handle the original file
            if placements > 0 {
                // Remove the original file after copying
                let _ = fs::remove_file(&file_hash.path);
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

/// Calculate the destination path for a ROM
fn calculate_rom_path(
    rom_name: &str,
    game_name: &str,
    needs_folder: bool,
    rom_dir: &str,
    is_disk: bool,
) -> Result<PathBuf> {
    let new_path = if is_disk {
        // CHDs go in a subdirectory named after the disk
        let disk_dir = Path::new(rom_dir).join(game_name).join(rom_name);
        fs::create_dir_all(&disk_dir)?;
        disk_dir.join(format!("{}.chd", rom_name))
    } else if needs_folder {
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