// src/parser/mod.rs - Parser module root

pub mod xml;

use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::types::ParsedDat;

pub trait DatParser {
    fn parse(&self, path: &Path) -> Result<ParsedDat>;
}

/// Find the first .dat file in the current directory
pub fn find_dat_file() -> Result<PathBuf> {
    std::fs::read_dir(".")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("dat"))
                .unwrap_or(false)
        })
        .ok_or(crate::error::RomAuditError::NoDatFile)
}

/// Parse DAT file
pub fn parse_dat_file(path: &Path) -> Result<ParsedDat> {
    let parser = xml::XmlParser::new();
    parser.parse(path)
}