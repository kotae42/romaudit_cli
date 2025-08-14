// src/parser/xml.rs - XML/DAT parser for standard DAT files only

use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::collections::HashSet;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::Result;
use crate::types::{RomEntry, RomHashes, RomDb, ParsedDat};
use super::DatParser;

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
        
        // For large files, use a larger buffer
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
            println!("Parsing DAT file ({:.1} MB)...", file_size as f64 / 1_048_576.0);
        }

        loop {
            match reader.read_event_into(&mut buf)? {
                // Handle <game> tags (standard DAT format)
                Event::Start(e) if e.name().as_ref() == b"game" => {
                    current_game = String::new();
                    
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"name" {
                                current_game = attr.unescape_value()?.to_string();
                            }
                        }
                    }

                    if !current_game.is_empty() {
                        all_games.insert(current_game.clone());
                        in_game_tag = true;
                    }
                }

                Event::End(e) if e.name().as_ref() == b"game" => {
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

                // Handle self-closing DISK tags
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

                // Handle opening ROM tags (for non-self-closing format)
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

                // Handle closing ROM tags
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
        }

        Ok(ParsedDat {
            rom_db,
            all_games,
        })
    }
}