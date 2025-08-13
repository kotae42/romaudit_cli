// src/database/mod.rs - Database module

use std::fs::{self, File};
use std::collections::{BTreeMap};
use serde_json;

use crate::error::Result;
use crate::types::KnownRoms;

/// Load known ROMs from database file
pub fn load_known_roms(db_file: &str) -> Result<KnownRoms> {
    match File::open(db_file) {
        Ok(file) => {
            let value: serde_json::Value = serde_json::from_reader(&file)?;
            let mut known_roms = KnownRoms::new();

            if let Some(obj) = value.as_object() {
                for (game_name, roms_obj) in obj {
                    if let Some(roms) = roms_obj.as_object() {
                        for (hash, rom_name_val) in roms {
                            if let Some(rom_name) = rom_name_val.as_str() {
                                known_roms.entry(hash.clone())
                                    .or_insert_with(Vec::new)
                                    .push((game_name.clone(), rom_name.to_string()));
                            }
                        }
                    } else if let Some(game_val) = roms_obj.as_str() {
                        // Old format compatibility
                        known_roms.entry(game_name.clone())
                            .or_insert_with(Vec::new)
                            .push((game_val.to_string(), String::new()));
                    }
                }
            }

            Ok(known_roms)
        }
        Err(_) => Ok(KnownRoms::new()),
    }
}

/// Save known ROMs to database file
pub fn save_known_roms(known_roms: &KnownRoms, db_file: &str) -> Result<()> {
    // Use BTreeMaps for automatic sorting by key, which is more efficient
    // than manual sorting. Structure: game_name -> (hash -> rom_name)
    let mut games_map: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    for (hash, entries) in known_roms {
        for (game, rom) in entries {
            games_map
                .entry(game.clone())
                .or_default()
                .insert(hash.clone(), rom.clone());
        }
    }

    // Convert the BTreeMap structure directly to a serde_json::Value
    let result = serde_json::to_value(&games_map)?;

    // Write to temporary file first, then rename atomically
    let temp_file = format!("{}.tmp", db_file);
    let file = File::create(&temp_file)?;
    serde_json::to_writer_pretty(file, &result)?;
    fs::rename(temp_file, db_file)?;

    Ok(())
}