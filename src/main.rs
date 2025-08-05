// romaudit_cli - ROM Collection Management Tool
// Version 1.6.1
// Copyright (c) 2024 [Kotae42]
//
// This software is licensed for PERSONAL USE ONLY.
// Commercial use is strictly prohibited.
// See LICENSE file for full terms.

use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use sha1::Sha1;
use digest::Digest;
use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use serde_json;

// Configuration structure for flexible paths
#[derive(Debug, Clone)]
struct Config {
    rom_dir: String,
    logs_dir: String,
    db_file: String,
    duplicate_prefix: String,
    unknown_prefix: String,
    buffer_size: usize,
    stop_words: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rom_dir: "roms".to_string(),
            logs_dir: "logs".to_string(),
            db_file: "rom_db.json".to_string(),
            duplicate_prefix: "duplicates".to_string(),
            unknown_prefix: "unknown".to_string(),
            buffer_size: 1024 * 1024, // 1MB
            stop_words: vec![
                "the", "of", "and", "a", "an", "in", "on", "at", "to", "for"
            ].into_iter().map(String::from).collect(),
        }
    }
}

// ROM organization rules:
// 1. Games with multiple ROM files → always use folders (roms/[Game Name]/[ROM files])
// 2. Single ROM games where ROM name matches game name → directly in roms/[ROM file]
// 3. Single ROM games where ROM name differs from game name → use folder (roms/[Game Name]/[ROM file])
//    Example: Game "Memory (Japan)", ROM "MEMORY.ASF" → roms/Memory (Japan)/MEMORY.ASF
// 4. ROMs with internal paths (containing \ or /) → preserve folder structure

#[derive(Debug)]
enum RomAuditError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Xml(quick_xml::Error),
    NoDatFile,
    InvalidPath(String),
}

impl fmt::Display for RomAuditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RomAuditError::Io(e) => write!(f, "IO error: {}", e),
            RomAuditError::Json(e) => write!(f, "JSON error: {}", e),
            RomAuditError::Xml(e) => write!(f, "XML error: {}", e),
            RomAuditError::NoDatFile => write!(f, "No .dat file found in current directory"),
            RomAuditError::InvalidPath(p) => write!(f, "Invalid path: {}", p),
        }
    }
}

impl Error for RomAuditError {}

impl From<std::io::Error> for RomAuditError {
    fn from(error: std::io::Error) -> Self {
        RomAuditError::Io(error)
    }
}

impl From<serde_json::Error> for RomAuditError {
    fn from(error: serde_json::Error) -> Self {
        RomAuditError::Json(error)
    }
}

impl From<quick_xml::Error> for RomAuditError {
    fn from(error: quick_xml::Error) -> Self {
        RomAuditError::Xml(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RomEntry {
    name: String,
    game: String,
    hashes: RomHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RomHashes {
    sha1: Option<String>,
    md5: Option<String>,
    crc: Option<String>,
}

// Maps hash -> list of rom entries that share this hash
type RomDb = HashMap<String, Vec<RomEntry>>;
// Maps sha1 -> list of (game name, rom name) tuples for all satisfied ROMs
type KnownRoms = HashMap<String, Vec<(String, String)>>;

struct ScanResult {
    have: HashSet<String>,
    missing: HashSet<String>,
    duplicate: Vec<String>,
    unknown: Vec<String>,
    shared_roms: HashMap<String, Vec<String>>, // hash -> list of games that share this ROM (each has their own copy)
}

struct RomAuditor {
    rom_db: RomDb,
    all_games: HashSet<String>,
    known_roms: KnownRoms,
    config: Config,
}

impl RomAuditor {
    fn with_config(config: Config) -> Result<Self, RomAuditError> {
        let dat_path = Self::find_dat_file()?;
        let (rom_db, all_games) = Self::parse_dat(&dat_path)?;
        let known_roms = Self::load_known_roms(&config.db_file)?;

        Ok(RomAuditor {
            rom_db,
            all_games,
            known_roms,
            config,
        })
    }

    fn run(&mut self) -> Result<(), RomAuditError> {
        let results = self.analyze_folder(Path::new("."))?;
        self.save_known_roms()?;
        self.write_logs(&results)?;
        self.remove_empty_folders(Path::new("."))?;
        Ok(())
    }

    fn find_dat_file() -> Result<PathBuf, RomAuditError> {
        std::fs::read_dir(".")?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| p.extension().map(|ext| ext == "dat").unwrap_or(false))
            .ok_or(RomAuditError::NoDatFile)
    }

    fn parse_dat(dat_path: &Path) -> Result<(RomDb, HashSet<String>), RomAuditError> {
        let file = File::open(dat_path)?;
        let mut reader = Reader::from_reader(BufReader::new(file));

        let mut buf = Vec::new();
        let mut current_game = String::new();
        let mut rom_db = RomDb::new();
        let mut all_games = HashSet::new();
        let mut in_game_tag = false;

        // For handling non-self-closing ROM tags
        let mut current_rom_name = String::new();
        let mut current_rom_hashes = RomHashes {
            sha1: None,
            md5: None,
            crc: None,
        };
        let mut in_rom_tag = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                // Handle both <game> and <machine> tags
                Event::Start(e) if e.name().as_ref() == b"game" || e.name().as_ref() == b"machine" => {
                    current_game = e.attributes()
                        .filter_map(Result::ok)
                        .find(|a| a.key.as_ref() == b"name")
                        .and_then(|a| a.unescape_value().ok())
                        .map(|s| s.to_string())
                        .unwrap_or_default();

                    if !current_game.is_empty() {
                        all_games.insert(current_game.clone());
                        in_game_tag = true;
                    }
                }

                Event::End(e) if e.name().as_ref() == b"game" || e.name().as_ref() == b"machine" => {
                    in_game_tag = false;
                }

                // Handle self-closing ROM tags (No-Intro style)
                Event::Empty(e) if e.name().as_ref() == b"rom" && in_game_tag => {
                    let mut name = String::new();
                    let mut hashes = RomHashes {
                        sha1: None,
                        md5: None,
                        crc: None,
                    };

                    for attr in e.attributes().filter_map(Result::ok) {
                        match attr.key.as_ref() {
                            b"name" => name = attr.unescape_value()?.to_string(),
                            b"crc" => hashes.crc = Some(attr.unescape_value()?.to_lowercase()),
                            b"md5" => hashes.md5 = Some(attr.unescape_value()?.to_lowercase()),
                            b"sha1" => hashes.sha1 = Some(attr.unescape_value()?.to_lowercase()),
                            _ => {}
                        }
                    }

                    let rom_entry = RomEntry {
                        name: name.clone(),
                        game: current_game.clone(),
                        hashes: hashes.clone(),
                    };

                    // Store by all available hash types
                    if let Some(ref sha1) = hashes.sha1 {
                        rom_db.entry(sha1.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                    if let Some(ref md5) = hashes.md5 {
                        rom_db.entry(md5.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                    if let Some(ref crc) = hashes.crc {
                        rom_db.entry(crc.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                }

                // Handle opening ROM tags (MAME style)
                Event::Start(e) if e.name().as_ref() == b"rom" && in_game_tag => {
                    in_rom_tag = true;
                    current_rom_name.clear();
                    current_rom_hashes = RomHashes {
                        sha1: None,
                        md5: None,
                        crc: None,
                    };

                    for attr in e.attributes().filter_map(Result::ok) {
                        match attr.key.as_ref() {
                            b"name" => current_rom_name = attr.unescape_value()?.to_string(),
                            b"crc" => current_rom_hashes.crc = Some(attr.unescape_value()?.to_lowercase()),
                            b"md5" => current_rom_hashes.md5 = Some(attr.unescape_value()?.to_lowercase()),
                            b"sha1" => current_rom_hashes.sha1 = Some(attr.unescape_value()?.to_lowercase()),
                            _ => {}
                        }
                    }
                }

                // Handle closing ROM tags (MAME style)
                Event::End(e) if e.name().as_ref() == b"rom" && in_rom_tag => {
                    in_rom_tag = false;

                    let rom_entry = RomEntry {
                        name: current_rom_name.clone(),
                        game: current_game.clone(),
                        hashes: current_rom_hashes.clone(),
                    };

                    // Store by all available hash types
                    if let Some(ref sha1) = current_rom_hashes.sha1 {
                        rom_db.entry(sha1.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                    if let Some(ref md5) = current_rom_hashes.md5 {
                        rom_db.entry(md5.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                    if let Some(ref crc) = current_rom_hashes.crc {
                        rom_db.entry(crc.clone()).or_insert_with(Vec::new).push(rom_entry.clone());
                    }
                }

                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        Ok((rom_db, all_games))
    }

    fn calculate_hashes(path: &Path, buffer_size: usize) -> Result<(String, String, String), RomAuditError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; buffer_size];

        let mut crc = Crc32Hasher::new();
        let mut md5 = Md5::new();
        let mut sha1 = Sha1::new();

        loop {
            match reader.read(&mut buffer)? {
                0 => break,
                n => {
                    let chunk = &buffer[..n];
                    crc.update(chunk);
                    md5.update(chunk);
                    sha1.update(chunk);
                }
            }
        }

        Ok((
            hex::encode(sha1.finalize()),
            hex::encode(md5.finalize()),
            format!("{:08x}", crc.finalize()),
        ))
    }

    fn load_known_roms(db_file: &str) -> Result<KnownRoms, RomAuditError> {
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

    fn save_known_roms(&self) -> Result<(), RomAuditError> {
        // Group by game name for better organization
        let mut games_map: HashMap<String, Vec<(String, String)>> = HashMap::new();

        for (hash, entries) in &self.known_roms {
            for (game, rom) in entries {
                games_map.entry(game.clone())
                    .or_insert_with(Vec::new)
                    .push((hash.clone(), rom.clone()));
            }
        }

        // Sort games alphabetically
        let mut sorted_games: Vec<_> = games_map.into_iter().collect();
        sorted_games.sort_by(|a, b| a.0.cmp(&b.0));

        // Create final structure
        let mut result = serde_json::Map::new();
        for (game, mut roms) in sorted_games {
            // Sort ROMs within each game
            roms.sort_by(|a, b| a.1.cmp(&b.1));

            let rom_entries: serde_json::Map<String, serde_json::Value> = roms
                .into_iter()
                .map(|(hash, rom_name)| (hash, serde_json::Value::String(rom_name)))
                .collect();

            result.insert(game, serde_json::Value::Object(rom_entries));
        }

        // Write to temporary file first, then rename atomically
        let temp_file = format!("{}.tmp", self.config.db_file);
        let file = File::create(&temp_file)?;
        serde_json::to_writer_pretty(file, &result)?;
        fs::rename(temp_file, &self.config.db_file)?;

        Ok(())
    }

    fn collect_files_recursively(&self, dir: &Path) -> Result<Vec<PathBuf>, RomAuditError> {
        let mut files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let file_name = path.file_name()
                    .ok_or_else(|| RomAuditError::InvalidPath(path.to_string_lossy().to_string()))?;

                let file_name_str = file_name.to_string_lossy();

                if path.extension().unwrap_or_default() != "dat" &&
                   file_name_str != self.config.db_file &&
                   !file_name_str.ends_with(".tmp") &&
                   !self.is_generated_directory(&path) {
                    files.push(path);
                }
            } else if path.is_dir() {
                if !self.is_generated_directory(&path) {
                    files.extend(self.collect_files_recursively(&path)?);
                }
            }
        }

        Ok(files)
    }

    fn is_generated_directory(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();

            // Check if this is one of our generated directories
            if name_str == self.config.rom_dir ||
               name_str == self.config.logs_dir ||
               name_str.starts_with(&self.config.duplicate_prefix) ||
               name_str.starts_with(&self.config.unknown_prefix) {
                return true;
            }

            // Also check ancestors
            path.ancestors().any(|ancestor| {
                if let Some(ancestor_name) = ancestor.file_name() {
                    let ancestor_str = ancestor_name.to_string_lossy();
                    ancestor_str == self.config.rom_dir ||
                    ancestor_str == self.config.logs_dir ||
                    ancestor_str.starts_with(&self.config.duplicate_prefix) ||
                    ancestor_str.starts_with(&self.config.unknown_prefix)
                } else {
                    false
                }
            })
        } else {
            false
        }
    }

    fn analyze_folder(&mut self, scan_path: &Path) -> Result<ScanResult, RomAuditError> {
        let mut result = ScanResult {
            have: HashSet::new(),
            missing: self.all_games.clone(),
            duplicate: vec![],
            unknown: vec![],
            shared_roms: HashMap::new(),
        };

        // Build initial have set from known_roms
        for entries in self.known_roms.values() {
            for (game, _) in entries {
                result.have.insert(game.clone());
                result.missing.remove(game);
            }
        }

        fs::create_dir_all(&self.config.rom_dir)?;
        fs::create_dir_all(&self.config.logs_dir)?;

        let mut files = self.collect_files_recursively(scan_path)?;
        files.sort_by_key(|p| p.to_string_lossy().to_lowercase());

        let games_needing_folders = self.identify_games_needing_folders();

        let bar = ProgressBar::new(files.len() as u64);
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}"
            ).unwrap(),
        );

        let mut duplicate_dir = None;
        let mut unknown_dir = None;

        // Note: Each game gets its own copy of any file it needs, renamed according to the DAT.
        // Files are only treated as duplicates if all their intended destinations already exist.

        for file in files {
            let filename = file.file_name()
                .ok_or_else(|| RomAuditError::InvalidPath(file.to_string_lossy().to_string()))?
                .to_string_lossy()
                .to_string();

            let (sha1, md5, crc) = Self::calculate_hashes(&file, self.config.buffer_size)?;

            // Find all ROM entries matching this file's hashes
            let matching_entries = [&sha1, &md5, &crc]
                .iter()
                .filter_map(|hash| self.rom_db.get(*hash))
                .flatten()
                .cloned()
                .collect::<Vec<_>>();

            if !matching_entries.is_empty() {
                // This is a known ROM
                // Process all matching entries - each game that needs this file gets its own copy
                let mut first_placement = true;
                let mut source_path = file.clone();
                let mut placed_successfully = false;

                for rom_entry in &matching_entries {
                    let game_name = &rom_entry.game;
                    let needs_folder = games_needing_folders.contains(game_name);

                    // Use game folder for:
                    // 1. Games with multiple ROMs
                    // 2. Single ROM games where name doesn't match
                    //    Example: Game "Memory (Japan)", ROM "MEMORY.ASF"
                    //    Without folder: roms/MEMORY.ASF (incorrect)
                    //    With folder: roms/Memory (Japan)/MEMORY.ASF (correct)
                    // 3. ROMs with internal folder structure
                    let use_game_folder = needs_folder ||
                                          (rom_entry.name.contains('\\') || rom_entry.name.contains('/'));

                    let new_path = if use_game_folder {
                        if rom_entry.name.contains('\\') || rom_entry.name.contains('/') {
                            let mut path_parts = Path::new(&self.config.rom_dir).join(&rom_entry.game);
                            for part in rom_entry.name.split(&['\\', '/'][..]) {
                                path_parts = path_parts.join(part);
                            }
                            if let Some(parent) = path_parts.parent() {
                                fs::create_dir_all(parent)?;
                            }
                            path_parts
                        } else {
                            let game_dir = Path::new(&self.config.rom_dir).join(&rom_entry.game);
                            fs::create_dir_all(&game_dir)?;
                            game_dir.join(&rom_entry.name)
                        }
                    } else {
                        fs::create_dir_all(&self.config.rom_dir)?;
                        Path::new(&self.config.rom_dir).join(&rom_entry.name)
                    };

                    if new_path.exists() {
                        // File already exists at destination, skip this placement
                        continue;
                    }

                    if first_placement {
                        // Move the file for the first placement
                        fs::rename(&file, &new_path)?;
                        source_path = new_path.clone(); // Update source path for future copies
                        first_placement = false;
                        placed_successfully = true;
                    } else {
                        // Copy the file for subsequent placements
                        fs::copy(&source_path, &new_path)?;
                    }

                    // Update tracking
                    result.have.insert(game_name.clone());
                    result.missing.remove(game_name);

                    // Add to known ROMs
                    self.known_roms.entry(sha1.clone())
                        .or_insert_with(Vec::new)
                        .push((game_name.clone(), rom_entry.name.clone()));
                }

                // If we couldn't place the file anywhere (all destinations existed), treat as duplicate
                if !placed_successfully {
                    if duplicate_dir.is_none() {
                        duplicate_dir = Some(Self::create_next_folder(&self.config.duplicate_prefix)?);
                    }
                    let dup_path = duplicate_dir.as_ref().unwrap()
                        .join(&filename);
                    fs::rename(&file, &dup_path)?;
                    result.duplicate.push(filename.clone());
                }
            } else {
                // Unknown ROM - move to unknown folder
                if unknown_dir.is_none() {
                    unknown_dir = Some(Self::create_next_folder(&self.config.unknown_prefix)?);
                }
                let unk_path = unknown_dir.as_ref().unwrap()
                    .join(&filename);
                fs::rename(&file, &unk_path)?;
                result.unknown.push(filename.clone());
            }

            bar.set_message(filename);
            bar.inc(1);
        }

        bar.finish_with_message("Scan complete!");

        // Track shared ROMs after all files are processed
        for (hash, entries) in &self.known_roms {
            if entries.len() > 1 {
                let games: Vec<String> = entries.iter()
                    .map(|(game, _)| game.clone())
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();

                if games.len() > 1 {
                    result.shared_roms.insert(hash.clone(), games);
                }
            }
        }

        Ok(result)
    }

    fn identify_games_needing_folders(&self) -> HashSet<String> {
        let mut games_needing_folders = HashSet::new();

        // Check each game to determine if it needs a folder
        for game_name in &self.all_games {
            // Get actual ROM count for this game
            let actual_rom_count = self.get_actual_rom_count(game_name);

            // Two cases for needing folders:
            // 1. Games with multiple ROMs always get folders
            // 2. Single ROM games where ROM name doesn't match game name
            //    Example: Game "[BIOS] Play-Yan Micro Key File (Japan)"
            //             ROM "play_yanmicro.ini"
            //    Result: roms/[BIOS] Play-Yan Micro Key File (Japan)/play_yanmicro.ini
            if actual_rom_count > 1 {
                games_needing_folders.insert(game_name.clone());
            } else if actual_rom_count == 1 {
                // For single ROM games, check if the ROM name matches the game name
                // Find the ROM entry for this game
                for rom_entries in self.rom_db.values() {
                    if let Some(entry) = rom_entries.iter().find(|e| &e.game == game_name) {
                        if !self.is_rom_name_similar_to_game(game_name, &entry.name) {
                            games_needing_folders.insert(game_name.clone());
                        }
                        break;
                    }
                }
            }
        }

        games_needing_folders
    }

    fn get_actual_rom_count(&self, game_name: &str) -> usize {
        // Get the actual number of unique ROM files for a game
        // by collecting unique ROM names (not hash entries)
        let mut rom_names = HashSet::new();

        for rom_entries in self.rom_db.values() {
            for rom_entry in rom_entries {
                if rom_entry.game == game_name {
                    rom_names.insert(&rom_entry.name);
                }
            }
        }

        rom_names.len()
    }

    fn is_rom_name_similar_to_game(&self, game_name: &str, rom_name: &str) -> bool {
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
        // If the game name has spaces but the ROM name doesn't (or uses underscores/dots),
        // they're probably too different
        let game_has_spaces = game_name.contains(' ');
        let rom_has_spaces = rom_without_ext.contains(' ');
        let rom_has_separators = rom_without_ext.contains('_') || rom_without_ext.contains('.');

        if game_has_spaces && !rom_has_spaces && rom_has_separators {
            // Cases like "[BIOS] Play-Yan Micro Key File (Japan)" vs "play_yanmicro.ini"
            // These are too different - use folders
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
            let game_base = self.extract_base_name(game_name);
            let rom_base = self.extract_base_name(rom_name);

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
        let game_base = self.extract_base_name(game_name);
        let rom_base = self.extract_base_name(rom_name);

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
        let game_words = self.extract_significant_words(&game_base.to_lowercase());
        let rom_words = self.extract_significant_words(&rom_base.to_lowercase());

        if game_words.len() >= 2 && rom_words.len() >= 2 {
            let common_words: HashSet<_> = game_words.intersection(&rom_words).collect();
            let similarity_ratio = common_words.len() as f32 / game_words.len().min(rom_words.len()) as f32;

            return similarity_ratio >= 0.7; // Higher threshold for multi-word names
        }

        false
    }

    fn extract_base_name(&self, name: &str) -> String {
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

    fn extract_significant_words(&self, text: &str) -> HashSet<String> {
        // Split on non-alphanumeric characters and filter out configured stop words
        text.split(|c: char| !c.is_alphanumeric())
            .map(|s| s.trim())
            .filter(|s| s.len() > 2 && !self.config.stop_words.contains(&s.to_string()))
            .map(|s| s.to_string())
            .collect()
    }

    fn create_next_folder(prefix: &str) -> Result<PathBuf, RomAuditError> {
        for i in 1..1000 {
            let candidate = PathBuf::from(format!("{}{}", prefix, i));
            match fs::create_dir(&candidate) {
                Ok(_) => return Ok(candidate),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Err(RomAuditError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not create numbered folder after 1000 attempts"
        )))
    }

    fn remove_empty_folders(&self, dir: &Path) -> Result<(), RomAuditError> {
        let mut folders_to_check = Vec::new();
        self.collect_folders_recursively(dir, &mut folders_to_check)?;

        // Sort by depth (deepest first)
        folders_to_check.sort_by(|a, b| {
            b.components().count().cmp(&a.components().count())
        });

        for folder in folders_to_check {
            if self.is_generated_directory(&folder) {
                continue;
            }

            if Self::is_folder_empty(&folder)? {
                let _ = fs::remove_dir(&folder);
            }
        }

        Ok(())
    }

    fn collect_folders_recursively(&self, dir: &Path, folders: &mut Vec<PathBuf>) -> Result<(), RomAuditError> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && !self.is_generated_directory(&path) {
                folders.push(path.clone());
                self.collect_folders_recursively(&path, folders)?;
            }
        }
        Ok(())
    }

    fn is_folder_empty(dir: &Path) -> Result<bool, RomAuditError> {
        let mut entries = fs::read_dir(dir)?;
        Ok(entries.next().is_none())
    }

    fn write_logs(&self, results: &ScanResult) -> Result<(), RomAuditError> {
        let have_log = Path::new(&self.config.logs_dir).join("have.txt");
        let missing_log = Path::new(&self.config.logs_dir).join("missing.txt");
        let shared_log = Path::new(&self.config.logs_dir).join("shared.txt");

        // Write have log
        let mut have_file = File::create(&have_log)?;
        writeln!(have_file, "ROMs Found: {} / {}", results.have.len(), self.all_games.len())?;
        writeln!(have_file)?;
        let mut have_list: Vec<_> = results.have.iter().collect();
        have_list.sort();
        for name in have_list {
            writeln!(have_file, "{}", name)?;
        }

        // Write missing log
        let mut missing_file = File::create(&missing_log)?;
        writeln!(missing_file, "Missing ROMs: {} / {}", results.missing.len(), self.all_games.len())?;
        writeln!(missing_file)?;
        let mut missing_list: Vec<_> = results.missing.iter().collect();
        missing_list.sort();
        for name in missing_list {
            writeln!(missing_file, "{}", name)?;
        }

        // Write shared ROMs log if there are any
        if !results.shared_roms.is_empty() {
            let mut shared_file = File::create(&shared_log)?;
            writeln!(shared_file, "Shared ROMs (same file content used by multiple games - each has its own copy):")?;
            writeln!(shared_file, "===============================================================================")?;
            writeln!(shared_file)?;

            let mut shared_list: Vec<_> = results.shared_roms.iter().collect();
            shared_list.sort_by_key(|(hash, _)| *hash);

            for (hash, games) in &shared_list {
                writeln!(shared_file, "Hash: {}", hash)?;
                writeln!(shared_file, "Shared by {} games:", games.len())?;

                // Try to find the ROM name(s) for this hash
                let mut rom_names = HashSet::new();
                if let Some(entries) = self.known_roms.get(*hash) {
                    for (_, rom_name) in entries {
                        rom_names.insert(rom_name.clone());
                    }
                }

                if !rom_names.is_empty() {
                    writeln!(shared_file, "ROM name(s): {}", rom_names.iter().cloned().collect::<Vec<_>>().join(", "))?;
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
        }

        // Write games needing folders log if there are any
        let folders_log = Path::new(&self.config.logs_dir).join("folders.txt");
        let games_needing_folders = self.identify_games_needing_folders();
        if !games_needing_folders.is_empty() {
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
        }

        println!("Audit complete!");
        println!("Found: {} / {} ROMs ({:.1}%)",
            results.have.len(),
            self.all_games.len(),
            (results.have.len() as f64 / self.all_games.len() as f64) * 100.0
        );
        println!("Duplicates: {}, Unknown: {}", results.duplicate.len(), results.unknown.len());

        if !results.shared_roms.is_empty() {
            println!("Shared ROMs: {} (check {}/shared.txt for details)",
                results.shared_roms.len(), self.config.logs_dir);
        }

        if !games_needing_folders.is_empty() {
            println!("Games in folders: {} (check {}/folders.txt for details)",
                games_needing_folders.len(), self.config.logs_dir);
        }

        println!("Check the {}/ directory for detailed results.", self.config.logs_dir);

        Ok(())
    }
}

fn main() {
    // You can customize the configuration here if needed
    let config = Config::default();

    // Or create a custom configuration:
    // let mut config = Config::default();
    // config.rom_dir = "my_roms".to_string();
    // config.logs_dir = "my_logs".to_string();

    // Or load from a config file:
    // let config = load_config_from_file("config.toml").unwrap_or_default();

    match RomAuditor::with_config(config).and_then(|mut auditor| auditor.run()) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}