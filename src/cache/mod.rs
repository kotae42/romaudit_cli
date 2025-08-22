// src/cache/mod.rs - Hash cache for performance optimization

use std::collections::HashMap;
use std::fs::{File, metadata};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use blake3;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileInfo {
    pub path: PathBuf,
    pub sha1: String,
    pub md5: String,
    pub crc: String,
    pub size: u64,
    pub modified: SystemTime,
    pub cache_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HashCache {
    entries: HashMap<String, CachedFileInfo>,
    version: u32,
}

impl HashCache {
    const CACHE_VERSION: u32 = 1;
    const CACHE_FILE: &'static str = ".romaudit_cache.bin";
    
    pub fn new() -> Self {
        HashCache {
            entries: HashMap::new(),
            version: Self::CACHE_VERSION,
        }
    }
    
    /// Load cache from disk
    pub fn load() -> Result<Self> {
        let cache_path = Path::new(Self::CACHE_FILE);
        if !cache_path.exists() {
            return Ok(Self::new());
        }
        
        let file = File::open(cache_path)?;
        let mut reader = BufReader::new(file);
        
        match bincode::deserialize_from(&mut reader) {
            Ok(cache) => {
                let cache: HashCache = cache;
                if cache.version == Self::CACHE_VERSION {
                    Ok(cache)
                } else {
                    // Version mismatch, start fresh
                    Ok(Self::new())
                }
            }
            Err(_) => {
                // Cache corrupted or old format, start fresh
                Ok(Self::new())
            }
        }
    }
    
    /// Save cache to disk
    pub fn save(&self) -> Result<()> {
        let cache_path = Path::new(Self::CACHE_FILE);
        let file = File::create(cache_path)?;
        let mut writer = BufWriter::new(file);
        bincode::serialize_into(&mut writer, self)?;
        Ok(())
    }
    
    /// Generate a cache key for a file based on path, size, and modification time
    fn generate_cache_key(path: &Path, size: u64, modified: SystemTime) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update(&size.to_le_bytes());
        
        // Convert SystemTime to a stable representation
        if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
            hasher.update(&duration.as_secs().to_le_bytes());
            hasher.update(&duration.subsec_nanos().to_le_bytes());
        }
        
        hasher.finalize().to_hex().to_string()
    }
    
    /// Check if we have valid cached hashes for a file
    pub fn get(&self, path: &Path) -> Option<CachedFileInfo> {
        let meta = metadata(path).ok()?;
        let size = meta.len();
        let modified = meta.modified().ok()?;
        
        let cache_key = Self::generate_cache_key(path, size, modified);
        
        self.entries.get(&cache_key).cloned()
    }
    
    /// Store file hashes in cache
    pub fn insert(&mut self, path: &Path, sha1: String, md5: String, crc: String) -> Result<()> {
        let meta = metadata(path)?;
        let size = meta.len();
        let modified = meta.modified()?;
        
        let cache_key = Self::generate_cache_key(path, size, modified);
        
        let info = CachedFileInfo {
            path: path.to_path_buf(),
            sha1,
            md5,
            crc,
            size,
            modified,
            cache_key: cache_key.clone(),
        };
        
        self.entries.insert(cache_key, info);
        Ok(())
    }
    
    /// Remove stale entries (files that no longer exist)
    #[allow(dead_code)]
    pub fn cleanup(&mut self) {
        self.entries.retain(|_, info| {
            info.path.exists() && {
                if let Ok(meta) = metadata(&info.path) {
                    meta.len() == info.size && 
                    meta.modified().map(|m| m == info.modified).unwrap_or(false)
                } else {
                    false
                }
            }
        });
    }
    
    /// Get cache statistics
    #[allow(dead_code)]
    pub fn stats(&self) -> (usize, usize) {
        let total = self.entries.len();
        let valid = self.entries.values()
            .filter(|info| info.path.exists())
            .count();
        (total, valid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    
    #[test]
    fn test_cache_key_generation() {
        let path = Path::new("test.rom");
        let size = 1024;
        let time = SystemTime::now();
        
        let key1 = HashCache::generate_cache_key(path, size, time);
        let key2 = HashCache::generate_cache_key(path, size, time);
        
        assert_eq!(key1, key2);
        
        // Different size should give different key
        let key3 = HashCache::generate_cache_key(path, size + 1, time);
        assert_ne!(key1, key3);
    }
}