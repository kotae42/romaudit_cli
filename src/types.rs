// src/types.rs - Shared type definitions

use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomEntry {
    pub name: String,
    pub game: String,
    pub hashes: RomHashes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RomHashes {
    pub sha1: Option<String>,
    pub md5: Option<String>,
    pub crc: Option<String>,
}

// Maps hash -> list of rom entries that share this hash
pub type RomDb = HashMap<String, Vec<RomEntry>>;

// Maps sha1 -> list of (game name, rom name) tuples for all satisfied ROMs
pub type KnownRoms = HashMap<String, Vec<(String, String)>>;

#[derive(Debug, Clone, PartialEq)]
pub enum DatType {
    NonMerged,  // Each game completely self-contained
    Split,      // Clones depend on parents
    Merged,     // Clones inside parent folders
    Standard,   // Non-MAME DATs (No-Intro, etc.)
}

#[derive(Debug)]
pub struct ScanResult {
    pub have: HashSet<String>,
    pub missing: HashSet<String>,
    pub duplicate: Vec<String>,
    pub unknown: Vec<String>,
    pub shared_roms: HashMap<String, Vec<String>>, // hash -> list of games that share this ROM
}

#[derive(Debug)]
pub struct ParsedDat {
    pub rom_db: RomDb,
    pub all_games: HashSet<String>,
    pub dat_type: DatType,
    pub parent_clone_map: HashMap<String, String>, // clone -> parent mapping
}

#[derive(Debug)]
#[allow(dead_code)]  // md5 and crc are collected but not directly read in current implementation
pub struct FileHash {
    pub path: std::path::PathBuf,
    pub sha1: String,
    pub md5: String,
    pub crc: String,
    pub matching_entries: Vec<RomEntry>,
}