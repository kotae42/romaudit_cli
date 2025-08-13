// src/parser/detector.rs - DAT type detector

use crate::types::DatType;

/// Detect if this is a MAME XML file based on specific identifiers
pub fn is_mame_xml(content: &str) -> bool {
    // Quick check for obvious MAME indicators
    if content.contains("<mame build=") || 
       content.contains("<!DOCTYPE mame") {
        return true;
    }
    
    // Check if it has game/machine tags with cloneof/romof attributes (MAME-specific)
    let has_game_tags = content.contains("<game ") || content.contains("<machine ");
    let has_clone_refs = content.contains("cloneof=") || content.contains("romof=");
    
    if has_game_tags && has_clone_refs {
        return true;
    }
    
    // Check for large number of games (MAME typically has thousands)
    let game_count = content.matches("<game ").count() + content.matches("<machine ").count();
    if game_count > 1000 {
        // Almost certainly MAME if it has 1000+ games
        return true;
    }
    
    // Check header/description for MAME mentions
    if let Some(header_end) = content.find("</header>") {
        let header = &content[..header_end];
        let header_lower = header.to_lowercase();
        if header_lower.contains("mame") || 
           header_lower.contains("multiple arcade machine emulator") ||
           header_lower.contains("arcade") && game_count > 100 {
            return true;
        }
    }
    
    // Additional check: Look for MAME-specific game names
    let mame_specific_games = [
        "neogeo", "decocass", "playch10", "megatech", "megaplay",
        "stvbios", "naomi", "naomi2", "awbios", "cpzn1", "cpzn2"
    ];
    
    for game in &mame_specific_games {
        if content.contains(&format!("name=\"{}\"", game)) {
            return true;
        }
    }
    
    false
}

/// Detect if filename indicates MAME
pub fn is_mame_filename(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower.contains("mame") || 
    lower.contains("arcade") ||
    (lower.contains("rom") && (lower.contains("merged") || lower.contains("split")))
}

/// Detect DAT type from filename or content
pub fn detect_dat_type(filename: &str, content: Option<&str>) -> DatType {
    // Check filename first
    let lower = filename.to_lowercase();
    if lower.contains("non-merged") || lower.contains("nonmerged") || lower.contains("non merged") {
        return DatType::NonMerged;
    }
    if lower.contains("split") && !lower.contains("non") {
        return DatType::Split;
    }
    if lower.contains("merged") && !lower.contains("non") && !lower.contains("split") {
        return DatType::Merged;
    }
    
    // Check content if provided
    if let Some(content) = content {
        let lower = content.to_lowercase();
        if lower.contains("non-merged") || lower.contains("nonmerged") {
            return DatType::NonMerged;
        }
        if lower.contains("split") && !lower.contains("non") {
            return DatType::Split;
        }
        if lower.contains("merged") && !lower.contains("non-merged") {
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
