// src/scanner/hasher.rs - Hash calculation

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crc32fast::Hasher as Crc32Hasher;
use md5::Md5;
use sha1::Sha1;
use digest::Digest;
use hex;

use crate::error::Result;

/// Calculate SHA1, MD5, and CRC32 hashes for a file
pub fn calculate_hashes(path: &Path, buffer_size: usize) -> Result<(String, String, String)> {
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