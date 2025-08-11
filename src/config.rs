// src/config.rs - Configuration module

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub rom_dir: String,
    pub logs_dir: String,
    pub db_file: String,
    pub duplicate_prefix: String,
    pub unknown_prefix: String,
    pub buffer_size: usize,
    pub stop_words: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            rom_dir: "roms".to_string(),
            logs_dir: "logs".to_string(),
            db_file: "rom_db.json".to_string(),
            duplicate_prefix: "duplicates".to_string(),
            unknown_prefix: "unknown".to_string(),
            buffer_size: 1024 * 1024, // 1MB
            stop_words: vec![
                "the", "of", "and", "a", "an", "in", "on", "at", "to", "for"
            ].into_iter().map(String::from).collect(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        // For now, just use defaults
        // Could be enhanced to load from config.toml if it exists
        Config::default()
    }
}