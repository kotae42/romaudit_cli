// src/parser/detector.rs - DAT type detector

use crate::types::DatType;

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