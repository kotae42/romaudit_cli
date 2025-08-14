// src/organizer/standard.rs - Simple organizer for non-MAME DATs.

use std::sync::atomic::{AtomicBool};
use std::sync::Arc;

use crate::config::Config;
use crate::error::Result;
use crate::types::{FileHash, KnownRoms, RomDb, ScanResult};
use super::plugin::OrganizerPlugin;

pub struct StandardOrganizer {
    _config: Config,
    _interrupted: Arc<AtomicBool>,
}

impl StandardOrganizer {
    pub fn new(config: Config, interrupted: Arc<AtomicBool>) -> Self {
        Self { _config: config, _interrupted: interrupted }
    }
}

impl OrganizerPlugin for StandardOrganizer {
    fn organize(&self, _file_hashes: Vec<FileHash>, _rom_db: &RomDb, _known_roms: &mut KnownRoms) -> Result<ScanResult> {
        println!("Standard organizer is not yet implemented.");
        Ok(ScanResult::default())
    }
}