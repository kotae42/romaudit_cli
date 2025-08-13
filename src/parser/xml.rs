// src/parser/xml.rs - XML/DAT parser (excerpt - just the parse function)

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::collections::{HashMap, HashSet};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::Result;
use crate::types::{RomEntry, RomHashes, RomDb, DatType, ParsedDat};
use super::DatParser;
use super::detector::is_mame_xml;

pub struct XmlParser;

impl XmlParser {
    pub fn new() -> Self {
        XmlParser
    }
}

impl DatParser for XmlParser {
    fn parse(&self, dat_path: &Path) -> Result<ParsedDat> {
        let file = File::open(dat_path)?;
        let file_size = file.metadata()?.len();
        
        // Read a larger sample to detect if it's a MAME XML (increased to 32KB)
        let mut sample = String::new();
        let mut file_for_sample = File::open(dat_path)?;
        let mut buffer = vec![0; 32768]; // 32KB should be enough to find MAME identifiers
        if let Ok(n) = file_for_sample.read(&mut buffer) {
            sample = String::from_utf8_lossy(&buffer[..n]).to_string();
        }
        
        // Also check filename for MAME
        let filename = dat_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check both content and filename for MAME indicators
        let is_mame = is_mame_xml(&sample) || 
                      filename.to_lowercase().contains("mame");
        
        if is_mame {
            println!("Detected MAME XML - using literal parsing mode");
        } else {
            println!("Detected standard DAT - using parent/clone handling");
        }
        
        // For large files (like MAME XMLs), use a larger buffer
        let buffer_size = if file_size > 10_000_000 {
            8192 * 1024  // 8MB buffer for files over 10MB
        } else {
            8192  // 8KB default
        };
        
        let mut reader = Reader::from_reader(BufReader::with_capacity(buffer_size, file));

        let mut buf = Vec::new();
        let mut current_game = String::new();
        let mut _current_parent = String::new();
        let mut rom_db = RomDb::new();
        let mut all_games = HashSet::new();
        let mut in_game_tag = false;
        let mut dat_type = DatType::Standard;
        let mut parent_clone_map = HashMap::new();
        let mut in_header = false;

        // For handling non-self-closing ROM tags
        let mut current_rom_name = String::new();
        let mut current_rom_hashes = RomHashes {
            sha1: None,
            md5: None,
            crc: None,
        };
        let mut in_rom_tag = false;

        // Progress indicator for large files
        let show_progress = file_size > 5_000_000;
        if show_progress {
            println!("Parsing large DAT/XML file ({:.1} MB)...", file_size as f64 / 1_048_576.0);
        }

        loop {
            match reader.read_event_into(&mut buf)? {
                // Handle header for DAT type detection
                Event::Start(e) if e.name().as_ref() == b"header" => {
                    in_header = true;
                }
                
                Event::End(e) if e.name().as_ref() == b"header" => {
                    in_header = false;
                }
                
                // Detect DAT type from name or description in header
                Event::Start(e) if in_header && (e.name().as_ref() == b"name" || e.name().as_ref() == b"description") => {
                    // Continue to read the text content
                }
                
                Event::Text(e) if in_header => {
                    let text = String::from_utf8_lossy(e.as_ref()).to_lowercase();
                    if text.contains("non-merged") {
                        dat_type = DatType::NonMerged;
                    } else if text.contains("split") {
                        dat_type = DatType::Split;
                    } else if text.contains("merged") && !text.contains("non-merged") {
                        dat_type = DatType::Merged;
                    }
                }

                // Handle both <game> and <machine> tags
                Event::Start(e) if e.name().as_ref() == b"game" || e.name().as_ref() == b"machine" => {
                    current_game = String::new();
                    let mut current_parent = String::new();
                    
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            match attr.key.as_ref() {
                                b"name" => current_game = attr.unescape_value()?.to_string(),
                                b"cloneof" => {
                                    // Only track parent/clone for non-MAME DATs
                                    if !is_mame {
                                        current_parent = attr.unescape_value()?.to_string();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if !current_game.is_empty() {
                        all_games.insert(current_game.clone());
                        // For MAME XMLs, don't track parent/clone relationships
                        if !is_mame && !current_parent.is_empty() {
                            parent_clone_map.insert(current_game.clone(), current_parent);
                        }
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

                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            match attr.key.as_ref() {
                                b"name" => name = attr.unescape_value()?.to_string(),
                                b"crc" => hashes.crc = Some(attr.unescape_value()?.to_lowercase()),
                                b"md5" => hashes.md5 = Some(attr.unescape_value()?.to_lowercase()),
                                b"sha1" => hashes.sha1 = Some(attr.unescape_value()?.to_lowercase()),
                                _ => {}
                            }
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

                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            match attr.key.as_ref() {
                                b"name" => current_rom_name = attr.unescape_value()?.to_string(),
                                b"crc" => current_rom_hashes.crc = Some(attr.unescape_value()?.to_lowercase()),
                                b"md5" => current_rom_hashes.md5 = Some(attr.unescape_value()?.to_lowercase()),
                                b"sha1" => current_rom_hashes.sha1 = Some(attr.unescape_value()?.to_lowercase()),
                                _ => {}
                            }
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

        if show_progress {
            println!("Parsed {} games with {} unique ROM hashes", all_games.len(), rom_db.len());
            if !is_mame && !parent_clone_map.is_empty() {
                println!("Found {} clone relationships", parent_clone_map.len());
            }
            if is_mame {
                println!("MAME XML mode: Each game treated as independent");
            }
        }

        Ok(ParsedDat {
            rom_db,
            all_games,
            dat_type,
            parent_clone_map,
            is_mame_dat: is_mame,
        })
    }
}
