// src/parser/mod.rs - Parser module root

pub mod xml;
pub mod detector;

use std::path::{Path, PathBuf};
use crate::error::Result;
use crate::types::ParsedDat;

pub trait DatParser {
    fn parse(&self, path: &Path) -> Result<ParsedDat>;
}

/// Find the first .dat or .xml file in the current directory
pub fn find_dat_file() -> Result<PathBuf> {
    std::fs::read_dir(".")?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| {
            p.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("dat") || ext.eq_ignore_ascii_case("xml"))
                .unwrap_or(false)
        })
        .ok_or(crate::error::RomAuditError::NoDatFile)
}

/// Parse any supported DAT file format
pub fn parse_dat_file(path: &Path) -> Result<ParsedDat> {
    // For now, we use XML parser for both .dat and .xml files
    // since most DAT files are XML-based
    let parser = xml::XmlParser::new();
    parser.parse(path)
}