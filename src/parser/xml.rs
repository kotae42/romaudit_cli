// src/parser/xml.rs - XML/DAT parser

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::collections::{HashMap, HashSet};

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::Result;
use crate::types::{RomEntry, RomHashes, RomDb, DatType, ParsedDat};
use super::DatParser;
use super::detector::{is_mame_xml, is_mame_filename, detect_dat_type};

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
        
        // Read a large sample to detect if it's a MAME XML (64KB should be enough)
        let mut sample = String::new();
        let mut file_for_sample = File::open(dat_path)?;
        let sample_size = std::cmp::min(65536, file_size as usize); // Read up to 64KB
        let mut buffer = vec![0; sample_size];
        if let Ok(n) = file_for_sample.read(&mut buffer) {
            sample = String::from_utf8_lossy(&buffer[..n]).to_string();
        }
        
        // Get filename for detection
        let filename = dat_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Detect DAT type from filename first
        let mut dat_type = detect_dat_type(filename, Some(&sample));
        
        // Check both content and filename for MAME indicators
        // If the filename contains MAME or the type is detected, it's likely MAME
        let is_mame = is_mame_xml(&sample) || 
                      is_mame_filename(filename) ||
                      dat_type != DatType::Standard;  // If a type was detected, it's likely MAME
        
        if is_mame {
            println!("Detected MAME XML - using literal parsing mode");
            println!("DAT type from filename: {:?}", dat_type);
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
        let mut rom_db = RomDb::new();
        let mut all_games = HashSet::new();
        let mut in_game_tag = false;
        let mut parent_clone_map = HashMap::new();

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
                // Handle header for additional DAT type detection if needed
                Event::Start(e) if e.name().as_ref() == b"header" => {
                    // Header detected - could parse header content if needed
                }
                
                Event::End(e) if e.name().as_ref() == b"header" => {
                    // End of header
                }
                
                // Check for DAT type info in header text elements
                Event::Text(e) => {
                    // If we haven't determined the type yet, check text content
                    if dat_type == DatType::Standard && is_mame {
                        let text = String::from_utf8_lossy(e.as_ref()).to_lowercase();
                        if text.contains("non-merged") || text.contains("nonmerged") {
                            dat_type = DatType::NonMerged;
                        } else if text.contains("split") && !text.contains("non") {
                            dat_type = DatType::Split;
                        } else if text.contains("merged") && !text.contains("non") {
                            dat_type = DatType::Merged;
                        }
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
                                b"cloneof" | b"romof" => {
                                    // Only track parent/clone relationships for MAME DATs
                                    if is_mame {
                                        current_parent = attr.unescape_value()?.to_string();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if !current_game.is_empty() {
                        all_games.insert(current_game.clone());
                        // For MAME XMLs, track parent/clone relationships
                        if is_mame && !current_parent.is_empty() {
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
                        is_disk: false,
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

                // Handle self-closing DISK tags (MAME CHDs)
                Event::Empty(e) if e.name().as_ref() == b"disk" && in_game_tag => {
                    let mut name = String::new();
                    let mut sha1 = None;

                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            match attr.key.as_ref() {
                                b"name" => name = attr.unescape_value()?.to_string(),
                                b"sha1" => sha1 = Some(attr.unescape_value()?.to_lowercase()),
                                _ => {}
                            }
                        }
                    }

                    if let Some(sha1_hash) = sha1 {
                        let rom_entry = RomEntry {
                            name,
                            game: current_game.clone(),
                            hashes: RomHashes { sha1: Some(sha1_hash.clone()), ..Default::default() },
                            is_disk: true,
                        };
                        rom_db.entry(sha1_hash).or_insert_with(Vec::new).push(rom_entry);
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
                        is_disk: false,
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
            if is_mame {
                if !parent_clone_map.is_empty() {
                    println!("Found {} parent/clone relationships.", parent_clone_map.len());
                }
            } else {
                println!("Standard DAT mode: No parent/clone handling will be applied.");
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
