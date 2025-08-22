// src/scanner/hasher_optimized.rs - Optimized hash calculation with memory-mapped I/O

use std::fs::{File, metadata};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use sha1::Sha1;
use digest::Digest;
use hex;
use memmap2::Mmap;

use crate::error::Result;
use crate::cache::HashCache;

/// Threshold for using memory-mapped I/O (10 MB)
const MMAP_THRESHOLD: u64 = 10 * 1024 * 1024;

/// Calculate hashes with caching and memory-mapped I/O optimization
pub fn calculate_hashes_cached(
    path: &Path, 
    buffer_size: usize, 
    cache: &mut HashCache
) -> Result<(String, String, String)> {
    // Check cache first
    if let Some(cached) = cache.get(path) {
        return Ok((cached.sha1, cached.md5, cached.crc));
    }
    
    // Calculate hashes
    let (sha1, md5, crc) = calculate_hashes_optimized(path, buffer_size)?;
    
    // Store in cache
    cache.insert(path, sha1.clone(), md5.clone(), crc.clone())?;
    
    Ok((sha1, md5, crc))
}

/// Calculate SHA1, MD5, and CRC32 hashes for a file with optimizations
pub fn calculate_hashes_optimized(path: &Path, buffer_size: usize) -> Result<(String, String, String)> {
    let file_size = metadata(path)?.len();
    
    // Use memory-mapped I/O for large files
    if file_size > MMAP_THRESHOLD {
        calculate_hashes_mmap(path)
    } else {
        calculate_hashes_buffered(path, buffer_size)
    }
}

/// Calculate hashes using memory-mapped I/O for large files
fn calculate_hashes_mmap(path: &Path) -> Result<(String, String, String)> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    let mut crc = Crc32Hasher::new();
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();
    
    // Process the entire memory-mapped file
    let data = &mmap[..];
    crc.update(data);
    md5.update(data);
    sha1.update(data);
    
    Ok((
        hex::encode(sha1.finalize()),
        hex::encode(md5.finalize()),
        format!("{:08x}", crc.finalize()),
    ))
}

/// Calculate hashes using buffered I/O for smaller files
fn calculate_hashes_buffered(path: &Path, buffer_size: usize) -> Result<(String, String, String)> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0; buffer_size];

    let mut crc = Crc32Hasher::new();
    let mut md5 = Md5::new();
    let mut sha1 = Sha1::new();

    loop {
        match reader.read(&mut buffer)? {
            0 => break,
            n => {
                let chunk = &buffer[..n];
                crc.update(chunk);
                md5.update(chunk);
                sha1.update(chunk);
            }
        }
    }

    Ok((
        hex::encode(sha1.finalize()),
        hex::encode(md5.finalize()),
        format!("{:08x}", crc.finalize()),
    ))
}

/// Async version of hash calculation for use with tokio
#[allow(dead_code)]
pub async fn calculate_hashes_async(
    path: PathBuf, 
    buffer_size: usize
) -> Result<(String, String, String)> {
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, BufReader};
    
    let file = File::open(&path).await?;
    let metadata = file.metadata().await?;
    let file_size = metadata.len();
    
    // For large files, use blocking thread pool with mmap
    if file_size > MMAP_THRESHOLD {
        tokio::task::spawn_blocking(move || {
            calculate_hashes_mmap(&path)
        }).await?
    } else {
        // Async buffered reading for smaller files
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; buffer_size];
        
        let mut crc = Crc32Hasher::new();
        let mut md5 = Md5::new();
        let mut sha1 = Sha1::new();
        
        loop {
            match reader.read(&mut buffer).await? {
                0 => break,
                n => {
                    let chunk = &buffer[..n];
                    crc.update(chunk);
                    md5.update(chunk);
                    sha1.update(chunk);
                }
            }
        }
        
        Ok((
            hex::encode(sha1.finalize()),
            hex::encode(md5.finalize()),
            format!("{:08x}", crc.finalize()),
        ))
    }
}

/// Batch hash calculation with parallel processing
#[allow(dead_code)]
pub async fn calculate_hashes_batch(
    paths: Vec<PathBuf>,
    buffer_size: usize,
    max_concurrent: usize,
) -> Vec<Result<(PathBuf, String, String, String)>> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;
    
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut tasks = Vec::new();
    
    for path in paths {
        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let path_clone = path.clone();
        
        let task = tokio::spawn(async move {
            let result = calculate_hashes_async(path_clone.clone(), buffer_size).await;
            drop(permit); // Release semaphore
            
            match result {
                Ok((sha1, md5, crc)) => Ok((path_clone, sha1, md5, crc)),
                Err(e) => Err(e),
            }
        });
        
        tasks.push(task);
    }
    
    // Collect all results
    let mut results = Vec::new();
    for task in tasks {
        match task.await {
            Ok(result) => results.push(result),
            Err(_) => results.push(Err(crate::error::RomAuditError::Custom(
                "Task join error".to_string()
            ))),
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;
    
    #[test]
    fn test_hash_calculation_small_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.rom");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Hello, World!").unwrap();
        
        let (sha1, md5, crc) = calculate_hashes_optimized(&file_path, 1024).unwrap();
        
        assert_eq!(sha1, "0a0a9f2a6772942557ab5355d76af442f8f65e01");
        assert_eq!(md5, "65a8e27d8879283831b664bd8b7f0ad4");
        assert_eq!(crc, "ec4ac3d0");
    }
    
    #[tokio::test]
    async fn test_async_hash_calculation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test_async.rom");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"Async test data").unwrap();
        
        let result = calculate_hashes_async(file_path.to_path_buf(), 1024).await;
        assert!(result.is_ok());
    }
}