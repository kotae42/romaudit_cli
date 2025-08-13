// src/parser/detector.rs - DAT type detector

use crate::types::DatType;

/// Detect if this is a MAME XML file based on specific identifiers
pub fn is_mame_xml(content: &str) -> bool {
    // Check for MAME-specific XML identifiers
    content.contains("<mame build=") || 
    content.contains("<!DOCTYPE mame [") ||
    content.contains("MAME ROM database") ||
    // Modern MAME uses <machine> instead of <game>
    (content.contains("<machine name=") && content.contains("romof=")) ||
    // Older MAME uses <game>
    (content.contains("<game name=") && content.contains("romof=")) ||
    // Check for cloneof attribute (common in MAME)
    content.contains("cloneof=") ||
    // Check header comments for MAME
    content.lines()
        .take(20)  // Check first 20 lines (increased from 10)
        .any(|line| {
            let lower = line.to_lowercase();
            lower.contains("mame") || 
            lower.contains("multiple arcade machine emulator")
        })
}

/// Detect DAT type from filename or content
#[allow(dead_code)]
pub fn detect_dat_type(filename: &str, content: Option<&str>) -> DatType {
    // Check filename first
    let lower = filename.to_lowercase();
    if lower.contains("non-merged") || lower.contains("nonmerged") {
        return DatType::NonMerged;
    }
    if lower.contains("split") {
        return DatType::Split;
    }
    if lower.contains("merged") && !lower.contains("non") {
        return DatType::Merged;
    }
    
    // Check content if provided
    if let Some(content) = content {
        let lower = content.to_lowercase();
        if lower.contains("non-merged") || lower.contains("nonmerged") {
            return DatType::NonMerged;
        }
        if lower.contains("split") {
            return DatType::Split;
        }
        if lower.contains("merged") && !lower.contains("non") {
            return DatType::Merged;
        }
    }
    
    // Default to Standard for non-MAME DATs
    DatType::Standard
}

/// Display name for DAT type
#[allow(dead_code)]
pub fn dat_type_name(dat_type: &DatType) -> &'static str {
    match dat_type {
        DatType::NonMerged => "Non-merged (self-contained games)",
        DatType::Split => "Split (clones depend on parents)",
        DatType::Merged => "Merged (clones in parent folders)",
        DatType::Standard => "Standard",
    }
}
