// src/organizer/plugin.rs
use crate::error::Result;
use crate::types::{FileHash, ScanResult, KnownRoms, RomDb};

pub trait OrganizerPlugin {
    fn organize(&self, file_hashes: Vec<FileHash>, rom_db: &RomDb, known_roms: &mut KnownRoms) -> Result<ScanResult>;
}